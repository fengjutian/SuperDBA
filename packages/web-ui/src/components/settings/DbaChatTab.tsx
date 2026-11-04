import {RotateCcw} from 'lucide-react';
import {useTranslation} from 'react-i18next';
import {useSettings} from '@/hooks/useSettings';
import {Button} from '@/components/ui/Button';
import {
    SettingButtonGroup,
    SettingNumberInput,
    SettingRow,
    SettingSection,
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

export function DbaChatTab() {
    const {t} = useTranslation();
    const {settings, updateSetting} = useSettings();
    const showSuggestions = settings.dbaChatShowSuggestions !== false;
    const callsPerBatch = settings.aiChatCallsPerBatch ?? DEFAULT_CALLS_PER_BATCH;
    const maxBatches = settings.aiChatMaxBatches ?? DEFAULT_MAX_BATCHES;
    const totalCalls = callsPerBatch * maxBatches;
    const activePreset: AnalysisPreset =
        (Object.entries(ANALYSIS_PRESETS).find(
            ([, preset]) =>
                preset.callsPerBatch === callsPerBatch && preset.maxBatches === maxBatches,
        )?.[0] as Exclude<AnalysisPreset, 'custom'> | undefined) ?? 'custom';

    const applyPreset = (preset: AnalysisPreset) => {
        if (preset === 'custom') return;
        const values = ANALYSIS_PRESETS[preset];
        void updateSetting('aiChatCallsPerBatch', values.callsPerBatch);
        void updateSetting('aiChatMaxBatches', values.maxBatches);
    };

    const resetDefaults = () => {
        void updateSetting('aiChatCallsPerBatch', DEFAULT_CALLS_PER_BATCH);
        void updateSetting('aiChatMaxBatches', DEFAULT_MAX_BATCHES);
        void updateSetting('dbaChatShowSuggestions', true);
        void updateSetting('dbaChatGenerateSuggestions', true);
        void updateSetting('dbaChatIncludeCurrentSql', true);
        void updateSetting('dbaChatIncludeLastResult', false);
    };

    return (
        <div className="max-w-3xl">
            <div className="mb-6">
                <h2 className="text-lg font-semibold text-primary">
                    {t('settings.dbaChat.title')}
                </h2>
                <p className="mt-1 text-sm text-muted">{t('settings.dbaChat.description')}</p>
            </div>

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
                            disabled={!showSuggestions}
                            onChange={(value) =>
                                void updateSetting('dbaChatGenerateSuggestions', value)
                            }
                        />
                    </SettingRow>
                </div>
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
            </SettingSection>

            <SettingSection
                title={t('settings.dbaChat.analysisSection')}
                description={t('settings.dbaChat.analysisSectionDesc')}
                action={
                    <Button variant="ghost" size="sm" onClick={resetDefaults}>
                        <RotateCcw size={13} aria-hidden="true" />
                        {t('settings.dbaChat.resetDefaults')}
                    </Button>
                }
            >
                <div className="divide-y divide-default">
                    <SettingRow
                        label={t('settings.dbaChat.analysisPreset')}
                        description={t('settings.dbaChat.analysisPresetDesc')}
                        vertical
                    >
                        <SettingButtonGroup
                            value={activePreset}
                            onChange={applyPreset}
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
        </div>
    );
}
