using System.Runtime.InteropServices;
using System.Text.Json;
using Switchboard.Models;

namespace Switchboard.Engine;

/// <summary>
/// Wrapper P/Invoke sur switchboard_ffi.dll (genere depuis le crate Rust `ffi`,
/// lui-meme un shim C ABI sur `switchboard-core`). Toutes les structures riches
/// passent en JSON plutot qu'en marshaling manuel de structs Rust/C#.
/// </summary>
public sealed class DevtoolEngine : IDisposable
{
    private const string Dll = "switchboard_ffi";

    [DllImport(Dll)] private static extern IntPtr switchboard_engine_new();
    [DllImport(Dll)] private static extern void switchboard_engine_free(IntPtr engine);
    [DllImport(Dll)] private static extern IntPtr switchboard_engine_list_apps_json(IntPtr engine);
    [DllImport(Dll)] private static extern void switchboard_engine_add_app_json(IntPtr engine, string draftJson);
    [DllImport(Dll)] private static extern void switchboard_engine_update_app_json(IntPtr engine, string id, string draftJson);
    [DllImport(Dll)] private static extern void switchboard_engine_remove_app(IntPtr engine, string id);
    [DllImport(Dll)] private static extern void switchboard_engine_start_app(IntPtr engine, string id);
    [DllImport(Dll)] private static extern void switchboard_engine_stop_app(IntPtr engine, string id);
    [DllImport(Dll)] private static extern void switchboard_engine_start_all(IntPtr engine);
    [DllImport(Dll)] private static extern void switchboard_engine_stop_all(IntPtr engine);
    [DllImport(Dll)] private static extern void switchboard_engine_clear_logs(IntPtr engine, string id);
    [DllImport(Dll)] private static extern bool switchboard_engine_export_logs(IntPtr engine, string id, string path);
    [DllImport(Dll)] private static extern void switchboard_string_free(IntPtr s);

    private readonly IntPtr _handle;

    public DevtoolEngine()
    {
        _handle = switchboard_engine_new();
    }

    public List<AppEntry> ListApps()
    {
        var raw = switchboard_engine_list_apps_json(_handle);
        if (raw == IntPtr.Zero) return [];
        try
        {
            var json = Marshal.PtrToStringUTF8(raw) ?? "[]";
            return JsonSerializer.Deserialize<List<AppEntry>>(json) ?? [];
        }
        finally
        {
            switchboard_string_free(raw);
        }
    }

    public void AddApp(AppDraftPayload draft) =>
        switchboard_engine_add_app_json(_handle, JsonSerializer.Serialize(draft));

    public void UpdateApp(string id, AppDraftPayload draft) =>
        switchboard_engine_update_app_json(_handle, id, JsonSerializer.Serialize(draft));

    public void RemoveApp(string id) => switchboard_engine_remove_app(_handle, id);

    public void StartApp(string id) => switchboard_engine_start_app(_handle, id);

    public void StopApp(string id) => switchboard_engine_stop_app(_handle, id);

    public void StartAll() => switchboard_engine_start_all(_handle);

    public void StopAll() => switchboard_engine_stop_all(_handle);

    public void ClearLogs(string id) => switchboard_engine_clear_logs(_handle, id);

    public bool ExportLogs(string id, string path) => switchboard_engine_export_logs(_handle, id, path);

    public void Dispose() => switchboard_engine_free(_handle);
}
