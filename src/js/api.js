// Tauri API calls
const { invoke } = window.__TAURI__.core;

// Stream management
export async function startStream(url, quality) {
    return await invoke('start_stream', { url, quality });
}

export async function stopStream() {
    return await invoke('stop_stream');
}

export async function getCurrentStream() {
    return await invoke('get_current_stream');
}

export async function isVlcRunning() {
    return await invoke('is_vlc_running');
}

// URL validation and processing
export async function validateTwitchUrl(url) {
    return await invoke('validate_twitch_url', { url });
}

export async function extractChannelName(url) {
    return await invoke('extract_channel_name', { url });
}

export async function normalizeTwitchUrl(url) {
    return await invoke('normalize_twitch_url', { url });
}

// Settings management
export async function loadSettings() {
    return await invoke('load_settings');
}

export async function saveSettings(settings) {
    return await invoke('save_settings', { newSettings: settings });
}

// Quick streams management
export async function addQuickStream(name, url) {
    return await invoke('add_quick_stream', { name, url });
}

export async function removeQuickStream(index) {
    return await invoke('remove_quick_stream', { index });
}

export async function updateQuickStreamStatus() {
    return await invoke('update_quick_stream_status');
}

// Stream status checking
export async function checkStreamStatus(url) {
    return await invoke('check_stream_status', { url });
}

// Application info
export async function getAppVersion() {
    return await invoke('get_app_version');
}