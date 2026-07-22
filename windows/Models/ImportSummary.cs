using System.Text.Json.Serialization;

namespace Switchboard.Models;

public class ImportSummary
{
    [JsonPropertyName("to_add")]
    public List<string> ToAdd { get; set; } = [];

    [JsonPropertyName("to_replace")]
    public List<string> ToReplace { get; set; } = [];

    [JsonPropertyName("invalid")]
    public int Invalid { get; set; }
}
