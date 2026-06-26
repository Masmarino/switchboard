using Microsoft.UI.Xaml;
using Microsoft.Windows.AppNotifications;

namespace Switchboard;

public partial class App : Application
{
    private Window? _window;

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        try
        {
            AppNotificationManager.Default.Register();
        }
        catch
        {
            // Best-effort : l'app reste utilisable meme si les notifications systeme
            // ne sont pas disponibles (ex: AUMID non enregistre pour un build unpackaged).
        }

        _window = new MainWindow();
        _window.Activate();
    }
}
