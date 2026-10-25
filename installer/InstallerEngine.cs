// AmbraMusic installer. GPL-3.0-or-later.
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Reflection;
using System.Security.Cryptography;
using System.Text;
using Microsoft.Win32;

namespace AmbraMusic.Setup {
    internal sealed class InstallerEngine {
        internal const string Marker = ".ambramusic-install";
        internal const string Manifest = "installed-files.sha256";
        const string ProductId = "AmbraMusic-C00A8AD5-5B75-4E24-858F-855F74F396E1";
        const string RegistryPath = @"Software\Microsoft\Windows\CurrentVersion\Uninstall\AmbraMusic";
        internal readonly string Target;
        readonly string parent;
        internal InstallerEngine(string target) {
            Target = Path.GetFullPath(target).TrimEnd(Path.DirectorySeparatorChar);
            parent = Path.GetDirectoryName(Target);
            if (String.IsNullOrEmpty(parent) || Target == Path.GetPathRoot(Target))
                throw new IOException("Cartella di installazione non valida.");
        }
        internal static string DefaultTarget {
            get { return Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "AmbraMusic"); }
        }
        internal static string Hash(Stream stream) {
            using (var sha = SHA256.Create()) return BitConverter.ToString(sha.ComputeHash(stream)).Replace("-", "").ToLowerInvariant();
        }
        static string HashFile(string path) { using (var s = File.OpenRead(path)) return Hash(s); }
        internal static Stream Payload() {
            var s = Assembly.GetExecutingAssembly().GetManifestResourceStream("AmbraMusic.Payload");
            if (s == null) throw new IOException("Pacchetto di installazione assente.");
            return s;
        }
        static string Child(string root, string relative) {
            if (String.IsNullOrWhiteSpace(relative) || Path.IsPathRooted(relative) || relative.Contains(":"))
                throw new IOException("Percorso non valido nel pacchetto.");
            string full = Path.GetFullPath(Path.Combine(root, relative.Replace('/', Path.DirectorySeparatorChar)));
            if (!full.StartsWith(root.TrimEnd(Path.DirectorySeparatorChar) + Path.DirectorySeparatorChar, StringComparison.OrdinalIgnoreCase))
                throw new IOException("Il pacchetto contiene un percorso esterno alla cartella prevista.");
            return full;
        }
        static void NoLinks(string path) {
            var current = new DirectoryInfo(path);
            while (current != null) {
                if (current.Exists && (current.Attributes & FileAttributes.ReparsePoint) != 0)
                    throw new IOException("Le cartelle di installazione non possono essere collegamenti o junction.");
                current = current.Parent;
            }
        }
        static List<string> Files(string root) {
            var result = new List<string>();
            var pending = new Stack<string>(); pending.Push(root);
            while (pending.Count > 0) {
                string dir = pending.Pop(); NoLinks(dir);
                foreach (var file in Directory.GetFiles(dir)) {
                    if ((File.GetAttributes(file) & FileAttributes.ReparsePoint) != 0) throw new IOException("Collegamento non previsto: " + file);
                    result.Add(file);
                }
                foreach (var child in Directory.GetDirectories(dir)) pending.Push(child);
            }
            return result;
        }
        bool Owned(string directory) {
            string marker = Path.Combine(directory, Marker);
            return File.Exists(marker) && File.ReadAllText(marker).Trim() == ProductId;
        }
        void RemoveOwnedTemporary(string path) {
            string full = Path.GetFullPath(path);
            if (!String.Equals(Path.GetDirectoryName(full), parent, StringComparison.OrdinalIgnoreCase) ||
                !Path.GetFileName(full).StartsWith(".AmbraMusic-", StringComparison.Ordinal) || !Owned(full))
                throw new IOException("Pulizia annullata: cartella temporanea non riconosciuta.");
            NoLinks(full); Files(full); Directory.Delete(full, true);
        }
        Dictionary<string, string> ReadManifest(string directory) {
            var entries = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            string path = Path.Combine(directory, Manifest);
            if (!File.Exists(path)) throw new IOException("Registro dei file installati assente.");
            foreach (string line in File.ReadAllLines(path)) {
                if (line.Length < 67 || line.Substring(64, 2) != "  ") throw new IOException("Registro dei file non valido.");
                string name = line.Substring(66).Replace('\\', '/'); Child(directory, name);
                string digest = line.Substring(0, 64);
                if (digest.Any(c => !Uri.IsHexDigit(c)) || entries.ContainsKey(name)) throw new IOException("Registro dei file non valido.");
                entries.Add(name, digest);
            }
            return entries;
        }
        internal void EnsureAppClosed() {
            int session; using (var current = Process.GetCurrentProcess()) session = current.SessionId;
            foreach (var process in Process.GetProcessesByName("AmbraMusic")) using (process) {
                try {
                    if (!process.HasExited && process.SessionId == session)
                        throw new InvalidOperationException("AmbraMusic è aperto. Scegli Esci dal menu dell’icona nell’area di notifica, poi riprova.");
                } catch (System.ComponentModel.Win32Exception) {
                    throw new IOException("Non posso verificare se AmbraMusic è aperto. Chiudilo prima di continuare.");
                }
            }
        }
        internal void Install(Action<int, string> progress) {
            NoLinks(parent); Directory.CreateDirectory(parent); EnsureAppClosed();
            if (Directory.Exists(Target) && !Owned(Target))
                throw new IOException("La cartella AmbraMusic esiste già e non è gestita da questo installer. Spostala o rinominala prima di continuare.");
            if (Directory.Exists(Target)) NoLinks(Target);
            progress(2, "Verifica del pacchetto…");
            using (var payload = Payload()) {
                if (!String.Equals(Hash(payload), BuildInfo.PayloadHash, StringComparison.OrdinalIgnoreCase))
                    throw new IOException("Verifica del pacchetto non riuscita. Scarica nuovamente l’installer dalla release GitHub.");
            }
            string stage = Path.Combine(parent, ".AmbraMusic-stage-" + Guid.NewGuid().ToString("N"));
            string backup = Path.Combine(parent, ".AmbraMusic-backup-" + Guid.NewGuid().ToString("N"));
            bool previousMoved = false, newMoved = false;
            Directory.CreateDirectory(stage); File.WriteAllText(Path.Combine(stage, Marker), ProductId);
            try {
                var manifest = new List<string>(); var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
                using (var payload = Payload()) using (var zip = new ZipArchive(payload, ZipArchiveMode.Read)) {
                    long total = zip.Entries.Sum(e => e.Length), done = 0;
                    foreach (var entry in zip.Entries) {
                        // Framework ZipFile may emit backslashes; store canonical names for upgrades.
                        string name = entry.FullName.Replace('\\', '/');
                        if (name.EndsWith("/")) continue;
                        if (!seen.Add(name) || String.Equals(name, Marker, StringComparison.OrdinalIgnoreCase) || String.Equals(name, Manifest, StringComparison.OrdinalIgnoreCase))
                            throw new IOException("Voce duplicata o riservata nel pacchetto.");
                        string dest = Child(stage, name); Directory.CreateDirectory(Path.GetDirectoryName(dest));
                        using (var input = entry.Open()) using (var output = new FileStream(dest, FileMode.CreateNew, FileAccess.Write)) {
                            var buffer = new byte[262144]; int count;
                            while ((count = input.Read(buffer, 0, buffer.Length)) > 0) {
                                output.Write(buffer, 0, count); done += count;
                                progress(5 + (int)(done * 80 / Math.Max(1, total)), "Installazione dei file…");
                            }
                        }
                        manifest.Add(HashFile(dest) + "  " + name);
                    }
                }
                foreach (var required in new[] { "AmbraMusic.exe", "libmpv-2.dll", "vcruntime140.dll", "vcruntime140_1.dll", "Uninstall.exe", "LICENSE", "THIRD-PARTY.md" })
                    if (!File.Exists(Path.Combine(stage, required))) throw new IOException("File necessario assente: " + required);
                // Preserve files added by the user; managed files are replaced by the new payload.
                if (Directory.Exists(Target)) {
                    var managed = ReadManifest(Target);
                    foreach (string old in Files(Target)) {
                        string rel = old.Substring(Target.Length + 1).Replace('\\', '/');
                        if (rel == Marker || rel == Manifest || managed.ContainsKey(rel)) continue;
                        string dest = Child(stage, rel);
                        if (File.Exists(dest)) throw new IOException("Un file aggiunto alla cartella è in conflitto con il nuovo pacchetto: " + rel);
                        Directory.CreateDirectory(Path.GetDirectoryName(dest)); File.Copy(old, dest, false);
                    }
                }
                File.WriteAllLines(Path.Combine(stage, Manifest), manifest, new UTF8Encoding(false));
                EnsureAppClosed(); progress(90, "Aggiornamento di AmbraMusic…");
                if (Directory.Exists(Target)) { Directory.Move(Target, backup); previousMoved = true; }
                Directory.Move(stage, Target); newMoved = true;
            } catch {
                if (newMoved) { Directory.Move(Target, stage); newMoved = false; }
                if (previousMoved) { Directory.Move(backup, Target); previousMoved = false; }
                throw;
            } finally {
                if (Directory.Exists(stage)) RemoveOwnedTemporary(stage);
            }
            if (previousMoved) {
                try { RemoveOwnedTemporary(backup); }
                catch (IOException) { progress(94, "Aggiornamento completato; una copia precedente resta nella cartella Programmi dell’utente."); }
            }
            progress(96, "File installati.");
        }
        internal int Uninstall(Action<int, string> progress) {
            if (!Directory.Exists(Target) || !Owned(Target)) throw new IOException("Installazione AmbraMusic non riconosciuta.");
            NoLinks(Target); EnsureAppClosed();
            var entries = ReadManifest(Target); int done = 0, retained = 0;
            foreach (var entry in entries) {
                string file = Child(Target, entry.Key); NoLinks(Path.GetDirectoryName(file));
                if (File.Exists(file)) {
                    if ((File.GetAttributes(file) & FileAttributes.ReparsePoint) != 0) throw new IOException("Collegamento non previsto nella cartella dell’app.");
                    if (String.Equals(HashFile(file), entry.Value, StringComparison.OrdinalIgnoreCase)) File.Delete(file);
                    else retained++;
                }
                progress(++done * 90 / Math.Max(1, entries.Count), "Rimozione dei file del programma…");
            }
            RemoveEmptyDirectories(Target);
            var remaining = Files(Target).Where(f => Path.GetFileName(f) != Marker && Path.GetFileName(f) != Manifest).ToList();
            if (remaining.Count == 0) {
                File.Delete(Path.Combine(Target, Manifest)); File.Delete(Path.Combine(Target, Marker)); Directory.Delete(Target, false);
            }
            return Math.Max(retained, remaining.Count);
        }
        static void RemoveEmptyDirectories(string directory) {
            foreach (string child in Directory.GetDirectories(directory)) {
                NoLinks(child); RemoveEmptyDirectories(child);
                if (!Directory.EnumerateFileSystemEntries(child).Any()) Directory.Delete(child, false);
            }
        }
        internal void Register(bool desktop) {
            string exe = Path.Combine(Target, "AmbraMusic.exe");
            string menu = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Programs), "AmbraMusic");
            Directory.CreateDirectory(menu);
            Shortcuts.Create(Path.Combine(menu, "AmbraMusic.lnk"), exe, Target);
            Shortcuts.Create(Path.Combine(menu, "Disinstalla AmbraMusic.lnk"), Path.Combine(Target, "Uninstall.exe"), Target);
            if (desktop) Shortcuts.Create(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory), "AmbraMusic.lnk"), exe, Target);
            using (var key = Registry.CurrentUser.CreateSubKey(RegistryPath)) {
                key.SetValue("DisplayName", "AmbraMusic"); key.SetValue("DisplayVersion", BuildInfo.Version);
                key.SetValue("Publisher", "Kyo89-sudo"); key.SetValue("InstallLocation", Target);
                key.SetValue("DisplayIcon", "\"" + exe + "\",0"); key.SetValue("UninstallString", "\"" + Path.Combine(Target, "Uninstall.exe") + "\"");
                key.SetValue("URLInfoAbout", "https://github.com/Kyo89-sudo/AmbraMusic");
                key.SetValue("NoModify", 1, RegistryValueKind.DWord); key.SetValue("NoRepair", 1, RegistryValueKind.DWord);
                long kb = Files(Target).Sum(f => new FileInfo(f).Length) / 1024;
                key.SetValue("EstimatedSize", (int)Math.Min(Int32.MaxValue, kb), RegistryValueKind.DWord);
            }
        }
        internal void Unregister() {
            string menu = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.Programs), "AmbraMusic");
            Shortcuts.RemoveIfTarget(Path.Combine(menu, "AmbraMusic.lnk"), Path.Combine(Target, "AmbraMusic.exe"));
            Shortcuts.RemoveIfTarget(Path.Combine(menu, "Disinstalla AmbraMusic.lnk"), Path.Combine(Target, "Uninstall.exe"));
            Shortcuts.RemoveIfTarget(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory), "AmbraMusic.lnk"), Path.Combine(Target, "AmbraMusic.exe"));
            if (Directory.Exists(menu) && !Directory.EnumerateFileSystemEntries(menu).Any()) Directory.Delete(menu, false);
            using (var key = Registry.CurrentUser.OpenSubKey(RegistryPath)) {
                if (key == null || !String.Equals(key.GetValue("InstallLocation") as string, Target, StringComparison.OrdinalIgnoreCase)) return;
            }
            Registry.CurrentUser.DeleteSubKeyTree(RegistryPath, false);
        }
    }
    internal static class Shortcuts {
        static object Open(string path, out object shell) {
            var type = Type.GetTypeFromProgID("WScript.Shell", true); shell = Activator.CreateInstance(type);
            return type.InvokeMember("CreateShortcut", BindingFlags.InvokeMethod, null, shell, new object[] { path });
        }
        internal static void Create(string path, string target, string workingDirectory) {
            object shell, shortcut = Open(path, out shell);
            try {
                var t = shortcut.GetType(); t.InvokeMember("TargetPath", BindingFlags.SetProperty, null, shortcut, new object[] { target });
                t.InvokeMember("WorkingDirectory", BindingFlags.SetProperty, null, shortcut, new object[] { workingDirectory });
                t.InvokeMember("IconLocation", BindingFlags.SetProperty, null, shortcut, new object[] { target + ",0" });
                t.InvokeMember("Save", BindingFlags.InvokeMethod, null, shortcut, new object[0]);
            } finally { System.Runtime.InteropServices.Marshal.FinalReleaseComObject(shortcut); System.Runtime.InteropServices.Marshal.FinalReleaseComObject(shell); }
        }
        internal static void RemoveIfTarget(string path, string target) {
            if (!File.Exists(path)) return;
            object shell, shortcut = Open(path, out shell);
            bool matches;
            try { matches = String.Equals(shortcut.GetType().InvokeMember("TargetPath", BindingFlags.GetProperty, null, shortcut, new object[0]) as string, target, StringComparison.OrdinalIgnoreCase); }
            finally { System.Runtime.InteropServices.Marshal.FinalReleaseComObject(shortcut); System.Runtime.InteropServices.Marshal.FinalReleaseComObject(shell); }
            if (matches) File.Delete(path);
        }
    }
}
