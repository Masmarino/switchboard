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
            // Best-effort : pas de notifications si l'AUMID n'est pas enregistre (build unpackaged).
        }

        _window = new MainWindow();
        _window.Activate();
    }
}
