/**
 * @superdba/plugin-api
 *
 * Public API surface for SuperDBA plugin UI extensions.
 * Plugin bundles import hooks and the `defineSlot` helper from this package;
 * the SuperDBA host injects the runtime implementation at load time.
 */

export {
    usePluginQuery,
    usePluginConnection,
    usePluginToast,
    usePluginSetting,
    usePluginTranslation,
    usePluginModal,
    usePluginTheme,
    openUrl,
} from './hooks';

export {getHost, assertHostCompat} from './host';

export {API_VERSION, MIN_HOST_VERSION} from './version';

export {defineSlot} from './slots';

export type {
    SlotName,
    SlotContext,
    SlotComponentProps,
    SlotContextMap,
    PluginSecretFieldState,
    TypedSlotProps,
    UIExtensionDeclaration,
} from './slots';

export type {
    PluginModalOptions,
    PluginConfig,
    ThemeColors,
    PluginQueryResult,
    PluginTranslator,
    SuperDBAHostApi,
    UsePluginQueryReturn,
    UsePluginConnectionReturn,
    UsePluginToastReturn,
    UsePluginSettingReturn,
    UsePluginModalReturn,
    UsePluginThemeReturn,
} from './types';
