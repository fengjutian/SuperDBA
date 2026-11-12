import {describe, expect, it} from 'vitest';
import type {TFunction} from 'i18next';
import {localizeChatToolSummary} from '@/utils/chatToolSummary';

const translations: Record<string, string> = {
    'chat.toolDenial.explainEstimatedRows':
        'EXPLAIN 预计最多需要处理 {{rows}} 行。请缩小筛选范围或检查更小的数据区间。',
    'chat.toolDenial.explainFullScan':
        'EXPLAIN 预计需要全表扫描 {{rows}} 行（{{relation}}）。请缩小筛选范围或添加合适的索引。',
    'chat.toolDenial.explainScanAndSort':
        'EXPLAIN 预计需要扫描并排序 {{rows}} 行。请缩小查询范围或使用有索引支持的排序。',
    'chat.toolDenial.explainCartesianJoin': 'EXPLAIN 检测到笛卡尔连接，已阻止 AI 查询。',
    'chat.toolDenial.explainUnsafeOutput': '无法安全评估 EXPLAIN 输出，已阻止 AI 查询。',
};

const t = ((key: string, values?: Record<string, string>) => {
    let message = translations[key] ?? key;
    for (const [name, value] of Object.entries(values ?? {})) {
        message = message.replace(`{{${name}}}`, value);
    }
    return message;
}) as TFunction;

describe('localizeChatToolSummary', () => {
    it('localizes an excessive EXPLAIN row estimate', () => {
        expect(
            localizeChatToolSummary(
                'AI query blocked because EXPLAIN estimates up to 5866949 rows. Narrow the filters or inspect a smaller range.',
                'zh-CN',
                t,
            ),
        ).toBe('EXPLAIN 预计最多需要处理 5,866,949 行。请缩小筛选范围或检查更小的数据区间。');
    });

    it('preserves summaries that do not have a localized pattern', () => {
        expect(localizeChatToolSummary('1 row returned', 'zh-CN', t)).toBe('1 row returned');
    });
});
