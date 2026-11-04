import {RotateCcw} from 'lucide-react';
import {useState} from 'react';
import {useTranslation} from 'react-i18next';
import {useNavigate} from 'react-router-dom';
import {useSettings} from '@/hooks/useSettings';
import {useToast} from '@/hooks/useToast';
import {Button} from '@/components/ui/Button';
import {ConfirmModal} from '@/components/modals/ConfirmModal';
import {sanitizeGeneratedStarters} from '@/utils/chatStarters';
import type {AiProvider} from '@/contexts/SettingsContext';
import {
    SettingButtonGroup,
    SettingNumberInput,
    SettingRow,
    SettingSection,
    SettingSlider,
    SettingToggle,
} from './SettingControls';

const DEFAULT_CALLS_PER_BATCH = 16;
const DEFAULT_MAX_BATCHES = 4;
const ANALYSIS_PRESETS = {
    quick: {callsPerBatch: 8, maxBatches: 2},
    standard: {callsPerBatch: DEFAULT_CALLS_PER_BATCH, maxBatches: DEFAULT_MAX_BATCHES},
    deep: {callsPerBatch: 24, maxBatches: 6},
} as const;

type AnalysisPreset = keyof typeof ANALYSIS_PRESETS | 'custom';

const DBA_PROVIDERS: AiProvider[] = [
    'openai',
    'anthropic',
    'deepseek',
    'minimax',
    'openrouter',
    'requesty',
    'custom-openai',
    'ollama',
];

export function DbaChatTab() {
    const {t} = useTranslation();
    const navigate = useNavigate();
    const {settings, updateSetting} = useSettings();
    const {showToast} = useToast();
    const [resetConfirmOpen, setResetConfirmOpen] = useState(false);
    const [customStartersDraft, setCustomStartersDraft] = useState(
        (settings.dbaChatCustomStarters ?? []).join('\n'),
    );
    const customStarterPreview = sanitizeGeneratedStarters(customStartersDraft.split(/\r?\n/));
    const showSuggestions = settings.dbaChatShowSuggestions !== false;
    const callsPerBatch = settings.aiChatCallsPerBatch ?? DEFAULT_CALLS_PER_BATCH;
    const maxBatches = settings.aiChatMaxBatches ?? DEFAULT_MAX_BATCHES;
    const totalCalls = callsPerBatch * maxBatches;
    const activePreset: AnalysisPreset =
        (Object.entries(ANALYSIS_PRESETS).find(
            ([, preset]) =>
                preset.callsPerBatch === callsPerBatch && preset.maxBatches === maxBatches,
        )?.[0] as Exclude<AnalysisPreset, 'custom'> | undefined) ?? 'custom';

    const applyPreset = async (preset: AnalysisPreset) => {
        if (preset === 'custom') return;
        const values = ANALYSIS_PRESETS[preset];
        try {
            await Promise.all([
                updateSetting('aiChatCallsPerBatch', values.callsPerBatch),
                updateSetting('aiChatMaxBatches', values.maxBatches),
            ]);
            showToast(t('settings.dbaChat.presetSaved'), {kind: 'success'});
        } catch (error) {
            showToast(t('settings.dbaChat.saveFailed', {error: String(error)}), {kind: 'error'});
        }
    };

    const saveCustomStarters = async () => {
        const previous = sanitizeGeneratedStarters(settings.dbaChatCustomStarters ?? []);
        setCustomStartersDraft(customStarterPreview.join('\n'));
        if (JSON.stringify(previous) === JSON.stringify(customStarterPreview)) return;
        try {
            await updateSetting('dbaChatCustomStarters', customStarterPreview);
            showToast(t('settings.dbaChat.customStartersSaved'), {kind: 'success'});
        } catch (error) {
            showToast(t('settings.dbaChat.saveFailed', {error: String(error)}), {kind: 'error'});
        }
    };

    const resetDefaults = async () => {
        setResetConfirmOpen(false);
        try {
            await Promise.all([
                updateSetting('aiChatCallsPerBatch', DEFAULT_CALLS_PER_BATCH),
                updateSetting('aiChatMaxBatches', DEFAULT_MAX_BATCHES),
                updateSetting('dbaChatShowSuggestions', true),
                updateSetting('dbaChatGenerateSuggestions', true),
                updateSetting('dbaChatIncludeCurrentSql', true),
                updateSetting('dbaChatIncludeLastResult', false),
                updateSetting('dbaChatSendOnEnter', true),
                updateSetting('dbaChatCustomStarters', []),
                updateSetting('dbaChatProvider', null),
                updateSetting('dbaChatModel', ''),
                updateSetting('dbaChatTemperature', 0),
                updateSetting('dbaChatMaxOutputTokens', 2048),
                updateSetting('dbaChatAutoContinue', false),
                updateSetting('dbaChatIncludeLastPlan', true),
                updateSetting('dbaChatProductionStrictMode', true),
                updateSetting('dbaChatMaskSensitiveResults', true),
            ]);
            setCustomStartersDraft('');
            showToast(t('settings.dbaChat.defaultsRestored'), {kind: 'success'});
        } catch (error) {
            showToast(t('settings.dbaChat.saveFailed', {error: String(error)}), {kind: 'error'});
        }
    };

    return (
        <div className="max-w-3xl">
            <ConfirmModal
                isOpen={resetConfirmOpen}
                onClose={() => setResetConfirmOpen(false)}
                title={t('settings.dbaChat.resetConfirmTitle')}
                message={t('settings.dbaChat.resetConfirmMessage')}
                confirmLabel={t('settings.dbaChat.resetDefaults')}
                onConfirm={() => void resetDefaults()}
                variant="warning"
            />
            <div className="mb-6">
                <h2 className="text-lg font-semibold text-primary">
                    {t('settings.dbaChat.title')}
                </h2>
                <p className="mt-1 text-sm text-muted">{t('settings.dbaChat.description')}</p>
            </div>

            <SettingSection
                title={t('settings.dbaChat.modelSection')}
                description={t('settings.dbaChat.modelSectionDesc')}
                action={
                    <Button
                        variant="secondary"
                        size="sm"
                        onClick={() => navigate('/settings?tab=ai')}
                    >
                        {t('settings.dbaChat.configureModel')}
                    </Button>
                }
            >
                <SettingRow
                    label={t('settings.dbaChat.currentModel')}
                    description={
                        settings.aiProvider
                            ? t('settings.dbaChat.currentModelValue', {
                                  provider: settings.aiProvider,
                                  model: settings.aiModel || t('settings.dbaChat.defaultModel'),
                              })
                            : t('settings.dbaChat.modelNotConfigured')
                    }
                >
                    <span
                        className={
                            settings.aiProvider
                                ? 'text-xs text-accent-success'
                                : 'text-xs text-accent-warning'
                        }
                    >
                        {settings.aiProvider
                            ? t('settings.dbaChat.modelReady')
                            : t('settings.dbaChat.modelRequired')}
                    </span>
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.dedicatedProvider')}
                    description={t('settings.dbaChat.dedicatedProviderDesc')}
                >
                    <select
                        value={settings.dbaChatProvider ?? ''}
                        onChange={(event) =>
                            void updateSetting(
                                'dbaChatProvider',
                                (event.target.value || null) as AiProvider | null,
                            )
                        }
                        className="rounded border border-default bg-base px-2 py-1.5 text-sm text-primary"
                    >
                        <option value="">{t('settings.dbaChat.inheritGlobal')}</option>
                        {DBA_PROVIDERS.map((provider) => (
                            <option key={provider} value={provider}>
                                {provider}
                            </option>
                        ))}
                    </select>
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.dedicatedModel')}
                    description={t('settings.dbaChat.dedicatedModelDesc')}
                >
                    <input
                        value={settings.dbaChatModel ?? ''}
                        onChange={(event) => void updateSetting('dbaChatModel', event.target.value)}
                        placeholder={settings.aiModel ?? t('settings.dbaChat.inheritGlobal')}
                        className="w-52 rounded border border-default bg-base px-2 py-1.5 text-sm text-primary"
                    />
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.temperature')}
                    description={t('settings.dbaChat.temperatureDesc')}
                >
                    <SettingSlider
                        min={0}
                        max={2}
                        step={0.1}
                        value={settings.dbaChatTemperature ?? 0}
                        formatValue={(value) => value.toFixed(1)}
                        onChange={(value) =>
                            void updateSetting(
                                'dbaChatTemperature',
                                Math.min(2, Math.max(0, value)),
                            )
                        }
                    />
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.maxOutputTokens')}
                    description={t('settings.dbaChat.maxOutputTokensDesc')}
                >
                    <SettingNumberInput
                        min={256}
                        max={32768}
                        step={256}
                        fallback={2048}
                        value={settings.dbaChatMaxOutputTokens ?? 2048}
                        onChange={(value) =>
                            void updateSetting(
                                'dbaChatMaxOutputTokens',
                                Math.min(32768, Math.max(256, value)),
                            )
                        }
                    />
                </SettingRow>
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.suggestionsSection')}
                description={t('settings.dbaChat.suggestionsSectionDesc')}
            >
                <div className="divide-y divide-default">
                    <SettingRow
                        label={t('settings.dbaChat.showSuggestions')}
                        description={t('settings.dbaChat.showSuggestionsDesc')}
                    >
                        <SettingToggle
                            checked={showSuggestions}
                            onChange={(value) =>
                                void updateSetting('dbaChatShowSuggestions', value)
                            }
                        />
                    </SettingRow>
                    <SettingRow
                        label={t('settings.dbaChat.generateSuggestions')}
                        description={t('settings.dbaChat.generateSuggestionsDesc')}
                    >
                        <SettingToggle
                            checked={settings.dbaChatGenerateSuggestions !== false}
                            disabled={!showSuggestions || customStarterPreview.length > 0}
                            onChange={(value) =>
                                void updateSetting('dbaChatGenerateSuggestions', value)
                            }
                        />
                    </SettingRow>
                    <SettingRow
                        label={t('settings.dbaChat.customStarters')}
                        description={t('settings.dbaChat.customStartersDesc')}
                        vertical
                    >
                        <textarea
                            value={customStartersDraft}
                            onChange={(event) => setCustomStartersDraft(event.target.value)}
                            onBlur={() => void saveCustomStarters()}
                            rows={4}
                            maxLength={500}
                            aria-label={t('settings.dbaChat.customStarters')}
                            placeholder={t('settings.dbaChat.customStartersPlaceholder')}
                            className="w-full resize-y rounded border border-default bg-base px-3 py-2 text-sm text-primary placeholder:text-muted focus:border-focus focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                        />
                        <div className="mt-2 flex items-center justify-between gap-3 text-xs text-muted">
                            <span>
                                {t('settings.dbaChat.customStartersCount', {
                                    count: customStarterPreview.length,
                                })}
                            </span>
                            {customStarterPreview.length > 0 && (
                                <span className="text-accent-warning">
                                    {t('settings.dbaChat.customStartersOverride')}
                                </span>
                            )}
                        </div>
                        {customStarterPreview.length > 0 && (
                            <div
                                className="mt-2 space-y-1"
                                aria-label={t('settings.dbaChat.customStartersPreview')}
                            >
                                {customStarterPreview.map((prompt) => (
                                    <div
                                        key={prompt}
                                        className="rounded border border-default bg-surface-secondary px-2 py-1.5 text-xs text-secondary"
                                    >
                                        {prompt}
                                    </div>
                                ))}
                            </div>
                        )}
                    </SettingRow>
                </div>
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.inputSection')}
                description={t('settings.dbaChat.inputSectionDesc')}
            >
                <SettingRow
                    label={t('settings.dbaChat.sendOnEnter')}
                    description={t('settings.dbaChat.sendOnEnterDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatSendOnEnter !== false}
                        onChange={(value) => void updateSetting('dbaChatSendOnEnter', value)}
                    />
                </SettingRow>
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.contextSection')}
                description={t('settings.dbaChat.contextSectionDesc')}
            >
                <SettingRow
                    label={t('settings.dbaChat.includeCurrentSql')}
                    description={t('settings.dbaChat.includeCurrentSqlDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatIncludeCurrentSql !== false}
                        onChange={(value) => void updateSetting('dbaChatIncludeCurrentSql', value)}
                    />
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.includeLastResult')}
                    description={t('settings.dbaChat.includeLastResultDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatIncludeLastResult === true}
                        onChange={(value) => void updateSetting('dbaChatIncludeLastResult', value)}
                    />
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.includeLastPlan')}
                    description={t('settings.dbaChat.includeLastPlanDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatIncludeLastPlan !== false}
                        onChange={(value) => void updateSetting('dbaChatIncludeLastPlan', value)}
                    />
                </SettingRow>
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.analysisSection')}
                description={t('settings.dbaChat.analysisSectionDesc')}
                action={
                    <Button variant="ghost" size="sm" onClick={() => setResetConfirmOpen(true)}>
                        <RotateCcw size={13} aria-hidden="true" />
                        {t('settings.dbaChat.resetDefaults')}
                    </Button>
                }
            >
                <div className="divide-y divide-default">
                    <SettingRow
                        label={t('settings.dbaChat.autoContinue')}
                        description={t('settings.dbaChat.autoContinueDesc')}
                    >
                        <SettingToggle
                            checked={settings.dbaChatAutoContinue === true}
                            onChange={(value) => void updateSetting('dbaChatAutoContinue', value)}
                        />
                    </SettingRow>
                    <SettingRow
                        label={t('settings.dbaChat.analysisPreset')}
                        description={t('settings.dbaChat.analysisPresetDesc')}
                        vertical
                    >
                        <SettingButtonGroup
                            value={activePreset}
                            onChange={(preset) => void applyPreset(preset)}
                            options={[
                                {value: 'quick', label: t('settings.dbaChat.presetQuick')},
                                {value: 'standard', label: t('settings.dbaChat.presetStandard')},
                                {value: 'deep', label: t('settings.dbaChat.presetDeep')},
                                ...(activePreset === 'custom'
                                    ? [
                                          {
                                              value: 'custom' as const,
                                              label: t('settings.dbaChat.presetCustom'),
                                          },
                                      ]
                                    : []),
                            ]}
                        />
                    </SettingRow>
                    <SettingRow
                        label={t('settings.dbaChat.callsPerBatch')}
                        description={t('settings.dbaChat.callsPerBatchDesc')}
                    >
                        <SettingNumberInput
                            min={1}
                            max={32}
                            fallback={1}
                            value={callsPerBatch}
                            onChange={(value) =>
                                void updateSetting(
                                    'aiChatCallsPerBatch',
                                    Math.min(32, Math.max(1, value)),
                                )
                            }
                        />
                    </SettingRow>
                    <SettingRow
                        label={t('settings.dbaChat.maxBatches')}
                        description={t('settings.dbaChat.maxBatchesDesc')}
                    >
                        <SettingNumberInput
                            min={1}
                            max={8}
                            fallback={1}
                            value={maxBatches}
                            onChange={(value) =>
                                void updateSetting(
                                    'aiChatMaxBatches',
                                    Math.min(8, Math.max(1, value)),
                                )
                            }
                        />
                    </SettingRow>
                </div>
                <div className="mt-3 rounded border border-accent-info/40 bg-accent-info/10 px-3 py-2 text-xs text-accent-info">
                    {t('settings.dbaChat.callBudget', {count: totalCalls})}
                </div>
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.securitySection')}
                description={t('settings.dbaChat.securitySectionDesc')}
            >
                <SettingRow
                    label={t('settings.dbaChat.productionStrictMode')}
                    description={t('settings.dbaChat.productionStrictModeDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatProductionStrictMode !== false}
                        onChange={(value) =>
                            void updateSetting('dbaChatProductionStrictMode', value)
                        }
                    />
                </SettingRow>
                <SettingRow
                    label={t('settings.dbaChat.maskSensitiveResults')}
                    description={t('settings.dbaChat.maskSensitiveResultsDesc')}
                >
                    <SettingToggle
                        checked={settings.dbaChatMaskSensitiveResults !== false}
                        onChange={(value) =>
                            void updateSetting('dbaChatMaskSensitiveResults', value)
                        }
                    />
                </SettingRow>
            </SettingSection>
        </div>
    );
}
