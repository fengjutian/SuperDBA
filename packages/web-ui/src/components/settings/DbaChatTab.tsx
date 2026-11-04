import {RotateCcw} from 'lucide-react';
import {useTranslation} from 'react-i18next';
import {useSettings} from '@/hooks/useSettings';
import {Button} from '@/components/ui/Button';
import {SettingNumberInput, SettingRow, SettingSection, SettingToggle} from './SettingControls';

const DEFAULT_CALLS_PER_BATCH = 16;
const DEFAULT_MAX_BATCHES = 4;

export function DbaChatTab() {
    const {t} = useTranslation();
    const {settings, updateSetting} = useSettings();
    const showSuggestions = settings.dbaChatShowSuggestions !== false;

    const resetDefaults = () => {
        void updateSetting('aiChatCallsPerBatch', DEFAULT_CALLS_PER_BATCH);
        void updateSetting('aiChatMaxBatches', DEFAULT_MAX_BATCHES);
        void updateSetting('dbaChatShowSuggestions', true);
        void updateSetting('dbaChatGenerateSuggestions', true);
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
                        label={t('settings.dbaChat.callsPerBatch')}
                        description={t('settings.dbaChat.callsPerBatchDesc')}
                    >
                        <SettingNumberInput
                            min={1}
                            max={32}
                            fallback={1}
                            value={settings.aiChatCallsPerBatch ?? DEFAULT_CALLS_PER_BATCH}
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
                            value={settings.aiChatMaxBatches ?? DEFAULT_MAX_BATCHES}
                            onChange={(value) =>
                                void updateSetting(
                                    'aiChatMaxBatches',
                                    Math.min(8, Math.max(1, value)),
                                )
                            }
                        />
                    </SettingRow>
                </div>
            </SettingSection>
        </div>
    );
}
