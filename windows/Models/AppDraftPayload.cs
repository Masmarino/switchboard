using System.Text.Json.Serialization;

namespace Switchboard.Models;

public class AppDraftPayload
{
    [JsonPropertyName("name")]
    public string Name { get; set; } = "";

    [JsonPropertyName("working_dir")]
    public string WorkingDir { get; set; } = "";

    [JsonPropertyName("kind")]
    public string Kind { get; set; } = "raw";

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
}
