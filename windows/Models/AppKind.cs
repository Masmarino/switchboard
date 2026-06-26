namespace Switchboard.Models;

public enum AppKind
{
    Cargo,
    Npm,
    Dotnet,
    Maven,
    Python,
    Go,
    Raw,
}

public static class AppKindExtensions
{
    public static string ToFfiValue(this AppKind kind) => kind switch
    {
        AppKind.Cargo => "cargo",
        AppKind.Npm => "npm",
        AppKind.Dotnet => "dotnet",
        AppKind.Maven => "maven",
        AppKind.Python => "python",
        AppKind.Go => "go",
        _ => "raw",
    };

    public static string Label(this AppKind kind) => kind switch
    {
        AppKind.Cargo => "CARGO",
        AppKind.Npm => "NPM",
        AppKind.Dotnet => "DOTNET",
        AppKind.Maven => "MAVEN",
        AppKind.Python => "PYTHON",
        AppKind.Go => "GO",
        _ => "RAW",
    };
}
