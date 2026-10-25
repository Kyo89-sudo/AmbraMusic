// Handwritten AmbraMusic setup UI and uninstaller. GPL-3.0-or-later.
using System;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;

[assembly: AssemblyTitle("AmbraMusic Setup")]
[assembly: AssemblyCompany("Kyo89-sudo")]
[assembly: AssemblyProduct("AmbraMusic")]
[assembly: AssemblyVersion(AmbraMusic.Setup.BuildInfo.AssemblyVersion)]
[assembly: AssemblyFileVersion(AmbraMusic.Setup.BuildInfo.AssemblyVersion)]
[assembly: AssemblyCopyright("GPL-3.0-or-later")]
[assembly: System.Runtime.Versioning.TargetFramework(".NETFramework,Version=v4.8", FrameworkDisplayName = ".NET Framework 4.8")]

namespace AmbraMusic.Setup {
    internal static class Program {
        internal static readonly bool UninstallMode =
#if UNINSTALLER
            true;
#else
            false;
#endif
        [STAThread]
        static int Main(string[] args) {
            // Use .NET 4.8 path handling, including long per-user and staging paths.
            AppContext.SetSwitch("Switch.System.IO.UseLegacyPathHandling", false);
            AppContext.SetSwitch("Switch.System.IO.BlockLongPaths", false);
            try {
#if !UNINSTALLER
                if (args.Length == 2 && args[0] == "--self-test") return SelfTest.Run(args[1]);
#else
                // Run outside the app directory so the installed Uninstall.exe can be removed.
                if (!args.Contains("--worker")) {
                    string temp = Path.Combine(Path.GetTempPath(), "AmbraMusic-Uninstall-" + Guid.NewGuid().ToString("N") + ".exe");
                    File.Copy(Assembly.GetExecutingAssembly().Location, temp, false);
                    Process.Start(new ProcessStartInfo(temp, "--worker") { UseShellExecute = false });
                    return 0;
                }
#endif
                Application.EnableVisualStyles(); Application.SetCompatibleTextRenderingDefault(false);
                bool created;
                using (var mutex = new Mutex(false, @"Local\AmbraMusic.Setup", out created)) {
                    if (!created) { MessageBox.Show("L’installer di AmbraMusic è già aperto.", "AmbraMusic"); return 1; }
                    Application.Run(new SetupWindow());
                }
                return 0;
            } catch (Exception e) {
                MessageBox.Show(e.Message, "AmbraMusic — operazione non completata", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return 1;
            }
        }
        internal static string ResourceText(string name) {
            using (var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream(name)) {
                if (stream == null) return "Documento non disponibile.";
                using (var reader = new StreamReader(stream, Encoding.UTF8)) return reader.ReadToEnd();
            }
        }
    }
    internal sealed class AccentButton : Button {
        internal AccentButton() {
            FlatStyle = FlatStyle.Flat; FlatAppearance.BorderSize = 0; BackColor = Color.FromArgb(255, 128, 18);
            ForeColor = Color.FromArgb(35, 27, 22); Cursor = Cursors.Hand; Font = new Font("Segoe UI", 10, FontStyle.Bold);
        }
    }
    internal sealed class AmberProgress : Control {
        int value;
        internal int Value { get { return value; } set { this.value = Math.Max(0, Math.Min(100, value)); Invalidate(); AccessibilityNotifyClients(AccessibleEvents.ValueChange, -1); } }
        internal AmberProgress() { DoubleBuffered = true; Height = 8; AccessibleRole = AccessibleRole.ProgressBar; }
        protected override AccessibleObject CreateAccessibilityInstance() { return new ProgressAccessibility(this); }
        sealed class ProgressAccessibility : ControlAccessibleObject {
            readonly AmberProgress progress;
            internal ProgressAccessibility(AmberProgress control) : base(control) { progress = control; }
            public override string Value { get { return progress.Value.ToString() + "%"; } set { } }
        }
        protected override void OnPaint(PaintEventArgs e) {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using (var track = new SolidBrush(Color.FromArgb(222, 222, 228))) e.Graphics.FillRectangle(track, 0, 0, Width, Height);
            using (var fill = new SolidBrush(Color.FromArgb(255, 128, 18))) e.Graphics.FillRectangle(fill, 0, 0, Width * value / 100f, Height);
        }
    }
    internal sealed class SetupWindow : Form {
        readonly InstallerEngine engine = new InstallerEngine(InstallerEngine.DefaultTarget);
        readonly Label heading, description, status;
        readonly AccentButton action;
        readonly Button cancel;
        readonly CheckBox desktop, launch;
        readonly AmberProgress progress;
        bool busy, complete;
        internal SetupWindow() {
            Text = "AmbraMusic " + BuildInfo.Version + (Program.UninstallMode ? " — Disinstallazione" : " — Installazione");
            AutoScaleDimensions = new SizeF(96F, 96F); AutoScaleMode = AutoScaleMode.Dpi; ClientSize = new Size(620, 458); FormBorderStyle = FormBorderStyle.FixedSingle;
            MaximizeBox = false; StartPosition = FormStartPosition.CenterScreen; BackColor = Color.FromArgb(247, 247, 249);
            Font = new Font("Segoe UI", 10); ForeColor = Color.FromArgb(32, 32, 36);
            using (var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("AmbraMusic.Icon"))
                if (stream != null) Icon = new Icon(stream);

            var header = new Panel { Dock = DockStyle.Top, Height = 102, BackColor = Color.White };
            var logo = new PictureBox { Bounds = new Rectangle(28, 21, 48, 48), SizeMode = PictureBoxSizeMode.Zoom };
            using (var stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("AmbraMusic.Logo"))
                if (stream != null) using (var source = Image.FromStream(stream)) logo.Image = new Bitmap(source);
            header.Controls.Add(logo);
            header.Controls.Add(new Label { Text = "AmbraMusic", Font = new Font("Segoe UI", 19, FontStyle.Bold), AutoSize = true, Location = new Point(88, 17) });
            header.Controls.Add(new Label { Text = "Versione " + BuildInfo.Version + "  ·  Windows x64", AutoSize = true, ForeColor = Color.FromArgb(100, 100, 112), Location = new Point(91, 59) });

            var footer = new Panel { Dock = DockStyle.Bottom, Height = 76, BackColor = Color.White };
            var license = new Button { Text = "Licenza e crediti", FlatStyle = FlatStyle.Flat, Bounds = new Rectangle(23, 21, 148, 35), ForeColor = Color.FromArgb(95, 95, 108), Cursor = Cursors.Hand };
            license.FlatAppearance.BorderSize = 0; license.Click += (s, e) => ShowCredits();
            cancel = new Button { Text = "Annulla", Bounds = new Rectangle(356, 19, 104, 38), FlatStyle = FlatStyle.Flat };
            cancel.FlatAppearance.BorderColor = Color.FromArgb(219, 219, 226); cancel.Click += (s, e) => Close();
            action = new AccentButton { Text = Program.UninstallMode ? "Disinstalla" : (Directory.Exists(engine.Target) ? "Aggiorna" : "Installa"), Bounds = new Rectangle(472, 19, 120, 38) };
            action.Click += async (s, e) => await PerformAction(); footer.Controls.AddRange(new Control[] { license, cancel, action });

            var body = new Panel { Dock = DockStyle.Fill, Padding = new Padding(28, 20, 28, 16) };
            heading = new Label { Text = Program.UninstallMode ? "Disinstalla AmbraMusic" : (Directory.Exists(engine.Target) ? "Aggiorna AmbraMusic" : "Installa AmbraMusic"), Font = new Font("Segoe UI", 17, FontStyle.Bold), Dock = DockStyle.Top, Height = 40 };
            description = new Label { Text = Program.UninstallMode ? "Rimuove il programma e i collegamenti. Account, preferenze e cache rimangono sul PC." : "Client desktop YouTube Music per Windows. Il pacchetto include il motore audio e si installa per il tuo account.", Dock = DockStyle.Top, Height = 54, ForeColor = Color.FromArgb(90, 90, 104) };
            var destination = new TextBox { Text = "Destinazione: cartella Programmi del tuo account.", ReadOnly = true, BorderStyle = BorderStyle.None, BackColor = body.BackColor, Dock = DockStyle.Top, Height = 28, ForeColor = Color.FromArgb(110, 110, 122), TabStop = false, AccessibleName = "Cartella di installazione" };
            desktop = new CheckBox { Text = "Crea un collegamento sul desktop", Dock = DockStyle.Top, Height = 30, Visible = !Program.UninstallMode, Checked = false };
            launch = new CheckBox { Text = "Avvia AmbraMusic al termine", Dock = DockStyle.Top, Height = 30, Visible = !Program.UninstallMode, Checked = true };
            status = new Label { Text = Program.UninstallMode ? "Puoi reinstallare l’app in qualsiasi momento." : "Nessun account amministratore richiesto.", Dock = DockStyle.Top, Height = 44, Padding = new Padding(0, 12, 0, 0), ForeColor = Color.FromArgb(100, 100, 112) };
            progress = new AmberProgress { Dock = DockStyle.Top, Margin = new Padding(0, 8, 0, 0) };
            body.Controls.AddRange(new Control[] { progress, status, launch, desktop, destination, description, heading });
            Controls.Add(body); Controls.Add(footer); Controls.Add(header);
            AcceptButton = action; CancelButton = cancel;
            FormClosing += (s, e) => { if (busy) e.Cancel = true; };
        }
        void ShowCredits() {
            using (var dialog = new Form { Text = "AmbraMusic — Licenza e crediti", StartPosition = FormStartPosition.CenterParent, Size = new Size(620, 480), MinimizeBox = false }) {
                var text = new TextBox { Dock = DockStyle.Fill, Multiline = true, ReadOnly = true, ScrollBars = ScrollBars.Vertical, Font = new Font("Segoe UI", 9), Text = Program.ResourceText("AmbraMusic.License") + "\r\n\r\n" + Program.ResourceText("AmbraMusic.Credits") };
                var close = new Button { Dock = DockStyle.Bottom, Height = 38, Text = "Chiudi", DialogResult = DialogResult.OK };
                dialog.Controls.Add(text); dialog.Controls.Add(close); dialog.AcceptButton = close; dialog.CancelButton = close; dialog.ShowDialog(this);
            }
        }
        void Report(int value, string message) {
            if (IsDisposed) return;
            BeginInvoke(new Action(() => { progress.Value = value; status.Text = message; }));
        }
        async Task PerformAction() {
            if (complete) {
                if (!Program.UninstallMode && launch.Checked) {
                    try { Process.Start(new ProcessStartInfo(Path.Combine(engine.Target, "AmbraMusic.exe")) { WorkingDirectory = engine.Target, UseShellExecute = false }); }
                    catch (Exception e) { MessageBox.Show(this, e.Message, "Avvio non riuscito"); return; }
                }
                Close(); return;
            }
            busy = true; action.Enabled = false; cancel.Enabled = false; desktop.Enabled = false; launch.Enabled = false;
            try {
                bool createDesktop = desktop.Checked; string integrationWarning = null; int retained = 0;
                await Task.Run(() => {
                    if (Program.UninstallMode) retained = engine.Uninstall(Report);
                    else engine.Install(Report);
                });
                // Shell shortcut COM calls belong on the UI's STA thread.
                try { if (Program.UninstallMode) engine.Unregister(); else engine.Register(createDesktop); }
                catch (Exception e) { integrationWarning = e.Message; }
                progress.Value = 100; complete = true;
                heading.Text = Program.UninstallMode ? "AmbraMusic è stato rimosso." : "AmbraMusic è pronto.";
                description.Text = Program.UninstallMode ? "Account e preferenze sono stati conservati." : "Apri l’app e ritrova la tua musica. Puoi accedervi anche dal menu Start.";
                status.Text = retained > 0 ? "I file aggiunti o modificati nella cartella dell’app sono stati conservati." : "Operazione completata.";
                if (integrationWarning != null) {
                    status.Text = "File aggiornati; alcuni collegamenti o la registrazione di Windows richiedono una verifica.";
                    Log(integrationWarning);
                }
                action.Text = !Program.UninstallMode && launch.Checked ? "Avvia" : "Chiudi";
                cancel.Visible = false;
            } catch (Exception e) {
                heading.Text = "Operazione non completata"; description.Text = e.Message; status.Text = "Puoi riprovare oppure annullare."; progress.Value = 0;
                action.Text = "Riprova"; cancel.Enabled = true; desktop.Enabled = true; launch.Enabled = true; Log(e.ToString());
            } finally { busy = false; action.Enabled = true; }
        }
        static void Log(string message) {
            try {
                string dir = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "AmbraMusic", "Installer");
                Directory.CreateDirectory(dir); File.AppendAllText(Path.Combine(dir, "setup.log"), DateTime.UtcNow.ToString("O") + " " + message + Environment.NewLine);
            } catch (IOException) { } catch (UnauthorizedAccessException) { }
        }
    }
#if !UNINSTALLER
    internal static class SelfTest {
        internal static int Run(string reportPath) {
            var log = new StringBuilder(); string fixture = null;
            try {
                reportPath = Path.GetFullPath(reportPath); string parent = Path.GetDirectoryName(reportPath); Directory.CreateDirectory(parent);
                fixture = Path.Combine(parent, ".AmbraMusic-test-" + Guid.NewGuid().ToString("N")); Directory.CreateDirectory(fixture);
                var engine = new InstallerEngine(Path.Combine(fixture, "AmbraMusic"));
                engine.Install((p, s) => { });
                if (!File.Exists(Path.Combine(engine.Target, "AmbraMusic.exe"))) throw new Exception("Fresh install failed");
                log.AppendLine("PASS fresh installation and embedded payload SHA256");
                File.WriteAllText(Path.Combine(engine.Target, "user-added.txt"), "preserve me");
                engine.Install((p, s) => { });
                if (File.ReadAllText(Path.Combine(engine.Target, "user-added.txt")) != "preserve me") throw new Exception("Upgrade lost unknown file");
                log.AppendLine("PASS upgrade preserves user-added files");
                File.AppendAllText(Path.Combine(engine.Target, "README.md"), "modified by test");
                int retained = engine.Uninstall((p, s) => { });
                if (File.Exists(Path.Combine(engine.Target, "AmbraMusic.exe")) || retained < 2 || !File.Exists(Path.Combine(engine.Target, "README.md")))
                    throw new Exception("Selective uninstall failed");
                log.AppendLine("PASS uninstall removes managed files and preserves modified/unknown files");
                var refuse = new InstallerEngine(Path.Combine(fixture, "Unrelated")); Directory.CreateDirectory(refuse.Target);
                File.WriteAllText(Path.Combine(refuse.Target, "keep.txt"), "keep");
                bool rejected = false; try { refuse.Install((p, s) => { }); } catch (IOException) { rejected = true; }
                if (!rejected || File.ReadAllText(Path.Combine(refuse.Target, "keep.txt")) != "keep") throw new Exception("Unrelated directory was changed");
                log.AppendLine("PASS unrelated directory rejected without modification");
                string link = Path.Combine(fixture, "AmbraMusic.lnk");
                Shortcuts.Create(link, Path.Combine(fixture, "example.exe"), fixture);
                Shortcuts.RemoveIfTarget(link, Path.Combine(fixture, "other.exe"));
                if (!File.Exists(link)) throw new Exception("Unrelated shortcut removed");
                Shortcuts.RemoveIfTarget(link, Path.Combine(fixture, "example.exe"));
                if (File.Exists(link)) throw new Exception("Shortcut not removed");
                log.AppendLine("PASS shortcut creation and target-checked removal");
                log.AppendLine("No Start menu, desktop or uninstall registry entries modified; fixture: " + fixture);
                File.WriteAllText(reportPath, log.ToString()); return 0;
            } catch (Exception e) { log.AppendLine("FAIL " + e); File.WriteAllText(reportPath, log.ToString()); return 1; }
        }
    }
#endif
}
