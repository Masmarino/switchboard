using System.Text.Json.Serialization;

namespace Switchboard.Models;

public class AppEntry
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = "";

    [JsonPropertyName("name")]
    public string Name { get; set; } = "";

    [JsonPropertyName("working_dir")]
    public string WorkingDir { get; set; } = "";

    [JsonPropertyName("kind")]
    [JsonConverter(typeof(JsonStringEnumConverter))]
    public AppKind Kind { get; set; }

    [JsonPropertyName("command")]
    public string Command { get; set; } = "";

    [JsonPropertyName("url")]
    public string? Url { get; set; }

    [JsonPropertyName("env_vars")]
    public List<List<string>> EnvVars { get; set; } = [];

    [JsonPropertyName("auto_restart")]
    public bool AutoRestart { get; set; }

    [JsonPropertyName("start_order")]
    public int StartOrder { get; set; }

    [JsonPropertyName("status_label")]
    public string StatusLabel { get; set; } = "stopped";

    [JsonPropertyName("error")]
    public string? Error { get; set; }

    [JsonPropertyName("active")]
    public bool Active { get; set; }

    [JsonPropertyName("logs")]
    public List<string> Logs { get; set; } = [];

    [JsonPropertyName("logs_base_seq")]
    public ulong LogsBaseSeq { get; set; }

    [JsonPropertyName("logs_replace")]
    public bool LogsReplace { get; set; }

    [JsonPropertyName("healthy")]
    public bool? Healthy { get; set; }

    [JsonPropertyName("cpu_percent")]
    public double CpuPercent { get; set; }

    [JsonPropertyName("memory_mb")]
    public double MemoryMb { get; set; }

    public string EnvVarsText =>
        string.Join("\n", EnvVars.Where(p => p.Count == 2).Select(p => $"{p[0]}={p[1]}"));

    /// Inverse of EnvVarsText — kept alongside it so the CLE=valeur format is defined once.
    public static List<List<string>> ParseEnvVarsText(string text) =>
        text.Split('\n')
            .Select(line => line.Split('=', 2))
            .Where(parts => parts.Length == 2 && parts[0].Trim().Length > 0)
            .Select(parts => new List<string> { parts[0].Trim(), parts[1].Trim() })
            .ToList();

    public string Subtitle
    {
        get
        {
            if (!Active) return StatusLabel;
            var resource = $"{CpuPercent:F0}% CPU · {MemoryMb:F0} Mo";
            return Healthy switch
            {
                true => $"{StatusLabel} · ✓ healthy · {resource}",
                false => $"{StatusLabel} · ✗ ne répond pas · {resource}",
                null => $"{StatusLabel} · {resource}",
            };
        }
    }
}
