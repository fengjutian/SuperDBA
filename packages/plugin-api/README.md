# @superdba/plugin-api

Public API surface for [SuperDBA](https://github.com/SuperDBA/SuperDBA) plugin UI extensions.

Plugin bundles import hooks, the `defineSlot` helper, and shared types from this package. The SuperDBA host injects the actual runtime implementation — the published package ships as thin stubs, so your bundle stays small and the host remains the single source of truth.

## Install

```bash
npm install --save-dev @superdba/plugin-api
# or: pnpm add -D @superdba/plugin-api
```

Treat `@superdba/plugin-api` and `react` as Vite externals when building your IIFE bundle — the host provides both at runtime.

## Quick start — a typed slot component

```tsx
import { defineSlot, usePluginConnection } from "@superdba/plugin-api";

const MySlot = defineSlot(
  "row-editor-sidebar.field.after",
  ({ context }) => {
    const { driver } = usePluginConnection();
    // context.columnName is `string` (not `string | undefined`) because the
    // chosen slot always provides it.
    return <div>{driver} · {context.columnName}</div>;
  },
);

export default MySlot.component;
```

## Hooks

| Hook | Returns | Purpose |
|------|---------|---------|
| `usePluginQuery()` | `{ executeQuery, loading, error }` | Execute read-only queries on the active connection |
| `usePluginConnection()` | `{ connectionId, driver, schema }` | Active connection metadata |
| `usePluginToast()` | `{ showInfo, showError, showWarning }` | System notifications |
| `usePluginModal()` | `{ openModal, closeModal }` | Host-managed modal with custom content |
| `usePluginSetting(pluginId)` | `{ getSetting, setSetting, setSettings }` | Read/write the plugin's own settings |
| `usePluginTheme()` | `{ themeId, themeName, isDark, colors }` | Current theme tokens |
| `usePluginTranslation(pluginId)` | `t(key)` | Plugin-scoped i18next translator |
| `openUrl(url)` | `Promise<void>` | Open a URL in the system browser |

All hooks must run inside a React component loaded by SuperDBA. Calling them outside the host throws a clear error instead of failing silently.

## Compatibility

The package exports `API_VERSION` and `MIN_HOST_VERSION`. Declare `API_VERSION` as `api_version` on each manifest `ui_extensions` entry so the host can reject incompatible bundles before executing them. Plugin authors can also opt in to a fail-fast compatibility check at component entry:

```ts
import { assertHostCompat } from "@superdba/plugin-api";
assertHostCompat(); // throws if the running SuperDBA is older than MIN_HOST_VERSION
```

| Package version | Minimum SuperDBA | Notes |
|-----------------|-------------------|-------|
| `0.1.0`         | `0.1.0` | Initial release |
| `0.1.1`         | `0.1.0` | Typed slots and automatic manifest compatibility gate |

## Slot reference

See the full slot list and context contracts in [`plugins/PLUGIN_GUIDE.md` § 3b](https://github.com/SuperDBA/SuperDBA/blob/main/plugins/PLUGIN_GUIDE.md) and the typed `SlotContextMap` in `src/slots.ts`.

## License

Apache-2.0, same as SuperDBA.
