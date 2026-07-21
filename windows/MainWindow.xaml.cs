using System.Collections.ObjectModel;
using System.IO;
using Microsoft.UI;
using Microsoft.UI.Composition.SystemBackdrops;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using Microsoft.UI.Xaml.Shapes;
using Microsoft.Windows.AppNotifications;
using Windows.UI;
using Switchboard.Engine;
using Switchboard.Models;
using WinForms = System.Windows.Forms;

namespace Switchboard;

public sealed partial class MainWindow : Window
{
    private static readonly (AppKind Kind, string Label)[] KindOptions =
    [
        (AppKind.Cargo, "Cargo"),
        (AppKind.Npm, "Npm"),
        (AppKind.Dotnet, "Dotnet"),
        (AppKind.Maven, "Maven"),
        (AppKind.Python, "Python"),
        (AppKind.Go, "Go"),
        (AppKind.Raw, "Raw"),
    ];

    private readonly DevtoolEngine _engine = new();
    private readonly DispatcherTimer _timer = new();
    private WinForms.NotifyIcon? _trayIcon;
    private string? _selectedId;
    private List<AppEntry> _apps = [];
    private readonly Dictionary<string, string> _lastStatus = [];
    private ulong _lastSeenRevision;
    private ulong _sinceSeq;
    private readonly List<string> _selectedLogs = [];
    /// Mirrors the Rust engine's own MAX_LOG_LINES cap — without this, a client that stays
    /// caught up with the server never hits the "replace" fallback that would otherwise
    /// reset _selectedLogs, so it grows unbounded for the lifetime of the process.
    private const int MaxDisplayedLogLines = 5000;
    private string _logFilter = "";

    public MainWindow()
    {
        InitializeComponent();
        SystemBackdrop = new MicaBackdrop();
        SetupTrayIcon();

        _timer.Interval = TimeSpan.FromMilliseconds(200);
        _timer.Tick += (_, _) => Refresh();
        _timer.Start();

        // Closing the window (or "Quitter" in the tray menu, which also closes it) ends
        // the process without running Engine's own cleanup-on-drop reliably — the OS
        // reclaims the process before .NET finalizers run. Stop supervised app trees
        // explicitly so they don't survive as orphans holding their ports.
        Closed += (_, _) => { _engine.StopAll(); _trayIcon?.Dispose(); };

        RefreshNow();
    }

    /// <summary>
    /// WinUI3 (app non packagee) n'a pas d'API de tray icon native — NotifyIcon (WinForms)
    /// est l'approche standard dans ce cas. Non teste (pas de machine Windows disponible).
    /// </summary>
    private void SetupTrayIcon()
    {
        try
        {
            var iconPath = System.IO.Path.Combine(AppContext.BaseDirectory, "icon.ico");
            _trayIcon = new WinForms.NotifyIcon
            {
                Icon = File.Exists(iconPath) ? new System.Drawing.Icon(iconPath) : System.Drawing.SystemIcons.Application,
                Text = "Switchboard",
                Visible = true,
            };
            RebuildTrayMenu();
            _trayIcon.DoubleClick += (_, _) => this.Activate();
        }
        catch
        {
            // Best-effort : l'app reste utilisable sans icone de tray.
        }
    }

    private void RebuildTrayMenu()
    {
        if (_trayIcon is null) return;
        var menu = new WinForms.ContextMenuStrip();
        foreach (var app in _apps)
        {
            var item = new WinForms.ToolStripMenuItem($"{(app.Active ? "■" : "▶")}  {app.Name} — {app.StatusLabel}");
            item.Click += (_, _) =>
            {
                if (app.Active) _engine.StopApp(app.Id); else _engine.StartApp(app.Id);
                RefreshNow();
            };
            menu.Items.Add(item);
        }
        menu.Items.Add(new WinForms.ToolStripSeparator());
        var startAll = new WinForms.ToolStripMenuItem("Tout démarrer");
        startAll.Click += (_, _) => { _engine.StartAll(); RefreshNow(); };
        var stopAll = new WinForms.ToolStripMenuItem("Tout arrêter");
        stopAll.Click += (_, _) => { _engine.StopAll(); RefreshNow(); };
        menu.Items.Add(startAll);
        menu.Items.Add(stopAll);
        menu.Items.Add(new WinForms.ToolStripSeparator());
        var quit = new WinForms.ToolStripMenuItem("Quitter Switchboard");
        quit.Click += (_, _) => Microsoft.UI.Xaml.Application.Current.Exit();
        menu.Items.Add(quit);
        _trayIcon.ContextMenuStrip = menu;
    }

    private void RefreshNow()
    {
        _apps = _engine.ListApps(_selectedId, _sinceSeq);
        if (_selectedId is { } id)
        {
            var view = _apps.FirstOrDefault(a => a.Id == id);
            if (view is not null)
            {
                if (view.LogsReplace)
                {
                    _selectedLogs.Clear();
                    _selectedLogs.AddRange(view.Logs);
                }
                else if (view.Logs.Count > 0)
                {
                    _selectedLogs.AddRange(view.Logs);
                    var overflow = _selectedLogs.Count - MaxDisplayedLogLines;
                    if (overflow > 0)
                    {
                        _selectedLogs.RemoveRange(0, overflow);
                    }
                }
                _sinceSeq = view.LogsBaseSeq + (ulong)view.Logs.Count;
            }
        }
        NotifyNewFailures();
        RebuildTrayMenu();
        if (_selectedId is null)
        {
            _selectedId = _apps.FirstOrDefault()?.Id;
            if (_selectedId is not null)
            {
                // Just adopted a fallback selection (first launch, or the
                // previously-selected app was just deleted) — its log-tracking
                // state must start fresh, or a stale _sinceSeq left over from
                // whichever app was selected before would be diffed against this
                // different app's own sequence numbers on the next refresh and
                // silently show a truncated or empty log view.
                _sinceSeq = 0;
                _selectedLogs.Clear();
            }
        }

        AppListView.Items.Clear();
        foreach (var app in _apps)
        {
            AppListView.Items.Add(BuildRow(app));
        }

        var selected = _apps.FirstOrDefault(a => a.Id == _selectedId);
        if (selected is null)
        {
            DetailTitle.Text = "Switchboard";
            DetailSubtitle.Text = "Aucune app configurée";
            LogText.Text = "";
            return;
        }

        DetailTitle.Text = selected.Name;
        DetailSubtitle.Text = selected.Subtitle;
        RenderLogs();
    }

    /// <summary>
    /// Point d'entree du timer de poll : ne relance RefreshNow (fetch complet +
    /// reconstruction UI) que si la revision moteur a change depuis le dernier tick.
    /// </summary>
    private void Refresh()
    {
        var rev = _engine.Revision();
        if (rev != _lastSeenRevision)
        {
            _lastSeenRevision = rev;
            RefreshNow();
        }
    }

    private void RenderLogs()
    {
        if (_selectedLogs.Count == 0)
        {
            LogText.Text = "Pas encore de logs. Démarre l'app pour voir sa sortie ici.";
            return;
        }
        var lines = string.IsNullOrEmpty(_logFilter)
            ? _selectedLogs
            : _selectedLogs.Where(l => l.Contains(_logFilter, StringComparison.OrdinalIgnoreCase)).ToList();
        LogText.Text = string.Join("\n", lines);
        LogScroller.UpdateLayout();
        LogScroller.ChangeView(null, LogScroller.ScrollableHeight, null, true);
    }

    private void NotifyNewFailures()
    {
        foreach (var app in _apps)
        {
            _lastStatus.TryGetValue(app.Id, out var previous);
            _lastStatus[app.Id] = app.StatusLabel;
            if (app.StatusLabel == "failed" && previous != "failed")
            {
                var xml = $"""
                    <toast>
                      <visual>
                        <binding template="ToastGeneric">
                          <text>{app.Name} a crashé</text>
                          <text>{app.Error ?? "Le process s'est arrêté de manière inattendue."}</text>
                        </binding>
                      </visual>
                    </toast>
                    """;
                try
                {
                    AppNotificationManager.Default.Show(new AppNotification(xml));
                }
                catch
                {
                    // Best-effort : ne bloque pas l'UI si les notifications ne sont pas disponibles.
                }
            }
        }
    }

    private FrameworkElement BuildRow(AppEntry app)
    {
        var color = app.StatusLabel switch
        {
            "running" => Color.FromArgb(255, 48, 209, 88),
            "building" => Color.FromArgb(255, 255, 159, 10),
            "failed" => Color.FromArgb(255, 255, 69, 58),
            _ => Color.FromArgb(255, 142, 142, 147),
        };

        var dot = new Ellipse { Width = 9, Height = 9, Fill = new SolidColorBrush(color), Margin = new Thickness(0, 0, 8, 0) };

        var nameText = new TextBlock { Text = app.Name, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold, FontSize = 14 };
        var kindBadge = new Border
        {
            Background = new SolidColorBrush(Colors.Gray) { Opacity = 0.15 },
            CornerRadius = new CornerRadius(5),
            Padding = new Thickness(6, 1, 6, 1),
            Margin = new Thickness(8, 0, 0, 0),
            Child = new TextBlock { Text = app.Kind.Label(), FontSize = 10, FontFamily = new FontFamily("Consolas") },
        };

        var topRow = new StackPanel { Orientation = Orientation.Horizontal, VerticalAlignment = VerticalAlignment.Center };
        topRow.Children.Add(dot);
        topRow.Children.Add(nameText);
        topRow.Children.Add(kindBadge);

        var statusText = new TextBlock
        {
            Text = app.Error ?? app.Subtitle,
            FontSize = 11,
            FontFamily = new FontFamily("Consolas"),
            Foreground = new SolidColorBrush(app.Error is not null ? Colors.OrangeRed : Colors.Gray),
            TextWrapping = TextWrapping.Wrap,
        };

        var actions = new StackPanel { Orientation = Orientation.Horizontal, HorizontalAlignment = HorizontalAlignment.Right };

        if (!string.IsNullOrWhiteSpace(app.Url))
        {
            var openBtn = new Button { Content = "", FontFamily = new FontFamily("Segoe MDL2 Assets"), Margin = new Thickness(0, 0, 4, 0) };
            openBtn.Click += async (_, _) =>
            {
                await Windows.System.Launcher.LaunchUriAsync(new Uri(app.Url!));
            };
            actions.Children.Add(openBtn);
        }

        var editBtn = new Button { Content = "", FontFamily = new FontFamily("Segoe MDL2 Assets"), Margin = new Thickness(0, 0, 4, 0) };
        editBtn.Click += (_, _) => ShowAppDialog(app);
        actions.Children.Add(editBtn);

        var startBtn = new Button { Content = "▶", IsEnabled = !app.Active, Margin = new Thickness(0, 0, 4, 0) };
        startBtn.Click += (_, _) => { _engine.StartApp(app.Id); _selectedId = app.Id; _sinceSeq = 0; _selectedLogs.Clear(); RefreshNow(); };
        actions.Children.Add(startBtn);

        var stopBtn = new Button { Content = "■", IsEnabled = app.Active, Margin = new Thickness(0, 0, 4, 0) };
        stopBtn.Click += (_, _) => { _engine.StopApp(app.Id); RefreshNow(); };
        actions.Children.Add(stopBtn);

        var deleteBtn = new Button { Content = "🗑" };
        deleteBtn.Click += (_, _) => { _engine.RemoveApp(app.Id); if (_selectedId == app.Id) _selectedId = null; RefreshNow(); };
        actions.Children.Add(deleteBtn);

        var bottomRow = new Grid();
        bottomRow.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        bottomRow.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Grid.SetColumn(statusText, 0);
        Grid.SetColumn(actions, 1);
        bottomRow.Children.Add(statusText);
        bottomRow.Children.Add(actions);

        var container = new StackPanel { Spacing = 4, Margin = new Thickness(8, 6, 8, 6), Tag = app.Id };
        container.Children.Add(topRow);
        container.Children.Add(bottomRow);
        return container;
    }

    private void OnAppSelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (AppListView.SelectedItem is FrameworkElement el && el.Tag is string id)
        {
            _selectedId = id;
            _sinceSeq = 0;
            _selectedLogs.Clear();
            RefreshNow();
        }
    }

    private void OnStartAllClicked(object sender, RoutedEventArgs e)
    {
        _engine.StartAll();
        RefreshNow();
    }

    private void OnStopAllClicked(object sender, RoutedEventArgs e)
    {
        _engine.StopAll();
        RefreshNow();
    }

    private void OnClearLogsClicked(object sender, RoutedEventArgs e)
    {
        if (_selectedId is { } id) _engine.ClearLogs(id);
        _sinceSeq = 0;
        _selectedLogs.Clear();
        RefreshNow();
    }

    private async void OnExportLogsClicked(object sender, RoutedEventArgs e)
    {
        if (_selectedId is not { } id) return;
        var app = _apps.FirstOrDefault(a => a.Id == id);
        var picker = new Windows.Storage.Pickers.FileSavePicker
        {
            SuggestedFileName = $"{app?.Name ?? "logs"}.log",
        };
        picker.FileTypeChoices.Add("Fichier log", [".log", ".txt"]);
        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var file = await picker.PickSaveFileAsync();
        if (file is not null)
        {
            _engine.ExportLogs(id, file.Path);
        }
    }

    private async void OnAboutClicked(object sender, RoutedEventArgs e)
    {
        var panel = new StackPanel { Spacing = 10 };
        panel.Children.Add(new TextBlock
        {
            Text = "Démarre, supervise et orchestre tes process de dev locaux — quel que soit le langage.",
            TextWrapping = TextWrapping.Wrap,
        });
        panel.Children.Add(new TextBlock { Text = "Version 0.1.0", Opacity = 0.6, FontSize = 12 });

        var skolln = new HyperlinkButton { Content = "Développé par SkollN — skolln.com", NavigateUri = new Uri("https://skolln.com") };
        var alume = new HyperlinkButton { Content = "Découvre aussi Alume — alume.skolln.com", NavigateUri = new Uri("https://alume.skolln.com") };
        var source = new HyperlinkButton { Content = "Code source (GPLv3)", NavigateUri = new Uri("https://github.com/masmarino/switchboard") };
        panel.Children.Add(skolln);
        panel.Children.Add(alume);
        panel.Children.Add(source);

        var dialog = new ContentDialog
        {
            Title = "À propos de Switchboard",
            Content = panel,
            CloseButtonText = "Fermer",
            XamlRoot = Content.XamlRoot,
        };
        await dialog.ShowAsync();
    }

    private void OnLogFilterChanged(object sender, TextChangedEventArgs e)
    {
        _logFilter = LogFilterBox.Text;
        RefreshNow();
    }

    private void OnAddAppClicked(object sender, RoutedEventArgs e) => ShowAppDialog(null);

    private async void ShowAppDialog(AppEntry? existing)
    {
        var nameBox = new TextBox { PlaceholderText = "Nom", Text = existing?.Name ?? "" };
        var dirBox = new TextBox { PlaceholderText = "Dossier", Text = existing?.WorkingDir ?? "" };
        var browseDirBtn = new Button { Content = "Parcourir…", Margin = new Thickness(8, 0, 0, 0) };
        browseDirBtn.Click += async (_, _) =>
        {
            var folderPicker = new Windows.Storage.Pickers.FolderPicker();
            var pickerHwnd = WinRT.Interop.WindowNative.GetWindowHandle(this);
            WinRT.Interop.InitializeWithWindow.Initialize(folderPicker, pickerHwnd);
            var folder = await folderPicker.PickSingleFolderAsync();
            if (folder is not null)
            {
                dirBox.Text = folder.Path;
            }
        };
        var dirRow = new Grid();
        dirRow.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        dirRow.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
        Grid.SetColumn(dirBox, 0);
        Grid.SetColumn(browseDirBtn, 1);
        dirRow.Children.Add(dirBox);
        dirRow.Children.Add(browseDirBtn);
        var kindCombo = new ComboBox { ItemsSource = KindOptions.Select(k => k.Label).ToArray(), HorizontalAlignment = HorizontalAlignment.Stretch };
        kindCombo.SelectedIndex = Math.Max(0, Array.FindIndex(KindOptions, k => k.Kind == (existing?.Kind ?? AppKind.Cargo)));
        var commandBox = new TextBox { PlaceholderText = "Commande (npm/raw)", Text = existing?.Command ?? "" };
        var urlBox = new TextBox { PlaceholderText = "http://localhost:3000 (optionnel)", Text = existing?.Url ?? "" };
        var autoRestartToggle = new ToggleSwitch { IsOn = existing?.AutoRestart ?? false, OnContent = "", OffContent = "" };
        var startOrderBox = new NumberBox
        {
            Value = existing?.StartOrder ?? 0,
            Minimum = 0,
            Maximum = 99,
            SpinButtonPlacementMode = NumberBoxSpinButtonPlacementMode.Inline,
        };
        var envVarsBox = new TextBox
        {
            PlaceholderText = "CLE=valeur (une par ligne)",
            Text = existing?.EnvVarsText ?? "",
            AcceptsReturn = true,
            Height = 80,
            TextWrapping = TextWrapping.Wrap,
        };

        var generalCard = MakeSectionCard(
            "Général",
            MakeFieldRow("Nom", nameBox),
            MakeFieldRow("Dossier", dirRow),
            MakeFieldRow("Type", kindCombo),
            MakeFieldRow("Commande", commandBox));
        var execCard = MakeSectionCard(
            "Exécution",
            MakeFieldRow("URL", urlBox),
            MakeFieldRow("Auto-restart", autoRestartToggle),
            MakeFieldRow("Ordre de démarrage", startOrderBox));
        var advancedCard = MakeSectionCard(
            "Avancé",
            MakeFieldRow("Variables d'env", envVarsBox));

        var headerIcon = new Border
        {
            Width = 36,
            Height = 36,
            CornerRadius = new CornerRadius(18),
            Background = new SolidColorBrush(Color.FromArgb(255, 0x04, 0x09, 0x43)),
            Child = new TextBlock
            {
                Text = existing is not null ? "✎" : "+",
                Foreground = new SolidColorBrush(Colors.White),
                FontSize = 16,
                FontWeight = Microsoft.UI.Text.FontWeights.SemiBold,
                HorizontalAlignment = HorizontalAlignment.Center,
                VerticalAlignment = VerticalAlignment.Center,
            },
        };
        var headerTextPanel = new StackPanel { Spacing = 2 };
        headerTextPanel.Children.Add(new TextBlock
        {
            Text = existing is not null ? "Modifier l'app" : "Ajouter une app",
            FontSize = 18,
            FontWeight = Microsoft.UI.Text.FontWeights.Bold,
        });
        headerTextPanel.Children.Add(new TextBlock
        {
            Text = existing is not null ? "Mets à jour la configuration de cette app" : "Configure une nouvelle app à superviser",
            FontSize = 12,
            Opacity = 0.6,
        });
        var headerPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 12, Margin = new Thickness(0, 0, 0, 4) };
        headerPanel.Children.Add(headerIcon);
        headerPanel.Children.Add(headerTextPanel);

        var panel = new StackPanel { Spacing = 20 };
        panel.Children.Add(headerPanel);
        panel.Children.Add(generalCard);
        panel.Children.Add(execCard);
        panel.Children.Add(advancedCard);

        var dialog = new ContentDialog
        {
            // No Title: the custom header inside Content replaces the default dialog title bar.
            Content = new ScrollViewer { Content = panel, MaxHeight = 560 },
            PrimaryButtonText = existing is not null ? "Enregistrer" : "Ajouter",
            CloseButtonText = "Annuler",
            XamlRoot = Content.XamlRoot,
        };

        if (await dialog.ShowAsync() == ContentDialogResult.Primary)
        {
            var kind = KindOptions[Math.Max(0, kindCombo.SelectedIndex)].Kind.ToFfiValue();
            var envVars = envVarsBox.Text
                .Split('\n')
                .Select(line => line.Split('=', 2))
                .Where(parts => parts.Length == 2 && parts[0].Trim().Length > 0)
                .Select(parts => new List<string> { parts[0].Trim(), parts[1].Trim() })
                .ToList();

            var draft = new AppDraftPayload
            {
                Name = nameBox.Text.Trim(),
                WorkingDir = dirBox.Text.Trim(),
                Kind = kind,
                Command = commandBox.Text.Trim(),
                Url = string.IsNullOrWhiteSpace(urlBox.Text) ? null : urlBox.Text.Trim(),
                EnvVars = envVars,
                AutoRestart = autoRestartToggle.IsOn,
                StartOrder = double.IsNaN(startOrderBox.Value) ? 0 : (int)startOrderBox.Value,
            };

            if (existing is not null)
            {
                _engine.UpdateApp(existing.Id, draft);
            }
            else
            {
                _engine.AddApp(draft);
            }
            RefreshNow();
        }
    }

    private static readonly SolidColorBrush SectionTitleBrush = new(Color.FromArgb(255, 0x04, 0x09, 0x43));

    /// <summary>
    /// Wraps a group of field rows in a titled "card" — a rounded, tinted-background
    /// container with a bold navy section title above it, mirroring the macOS/Linux
    /// add-app dialog redesign (System Settings / GNOME PreferencesGroup style).
    /// </summary>
    private static StackPanel MakeSectionCard(string title, params FrameworkElement[] rows)
    {
        var titleBlock = new TextBlock
        {
            Text = title.ToUpperInvariant(),
            FontSize = 11,
            FontWeight = Microsoft.UI.Text.FontWeights.Bold,
            Foreground = SectionTitleBrush,
            Margin = new Thickness(0, 0, 0, 8),
        };

        var rowsPanel = new StackPanel { Spacing = 12 };
        foreach (var row in rows)
        {
            rowsPanel.Children.Add(row);
        }

        var card = new Border
        {
            Background = new SolidColorBrush(Colors.Gray) { Opacity = 0.08 },
            CornerRadius = new CornerRadius(10),
            Padding = new Thickness(14),
            Child = rowsPanel,
        };

        var container = new StackPanel { Spacing = 8 };
        container.Children.Add(titleBlock);
        container.Children.Add(card);
        return container;
    }

    /// <summary>
    /// A label + control row with a fixed-width label column, so every row's control
    /// lines up into a clean grid regardless of label length.
    /// </summary>
    private static Grid MakeFieldRow(string label, FrameworkElement control)
    {
        var grid = new Grid();
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(150) });
        grid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
        var labelBlock = new TextBlock { Text = label, VerticalAlignment = VerticalAlignment.Center };
        Grid.SetColumn(labelBlock, 0);
        Grid.SetColumn(control, 1);
        grid.Children.Add(labelBlock);
        grid.Children.Add(control);
        return grid;
    }
}
