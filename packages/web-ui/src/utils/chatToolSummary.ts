import type {TFunction} from 'i18next';

const ESTIMATED_ROWS =
    /^AI query blocked because EXPLAIN estimates up to (\d+) rows\. Narrow the filters or inspect a smaller range\.$/;
const FULL_SCAN =
    /^AI query blocked because EXPLAIN estimates a full scan of up to (\d+) rows \((.+)\)\. Narrow the filters or add an appropriate index\.$/;
const SCAN_AND_SORT =
    /^AI query blocked because EXPLAIN estimates a scan and sort of up to (\d+) rows\. Narrow the query or use an indexed order\.$/;

function formatRows(rows: string, language: string): string {
    return new Intl.NumberFormat(language).format(Number(rows));
}

export function localizeChatToolSummary(
    summary: string,
    language: string,
    t: TFunction,
): string {
    const estimatedRows = summary.match(ESTIMATED_ROWS);
    if (estimatedRows) {
        return t('chat.toolDenial.explainEstimatedRows', {
            rows: formatRows(estimatedRows[1], language),
        });
    }

    const fullScan = summary.match(FULL_SCAN);
    if (fullScan) {
        return t('chat.toolDenial.explainFullScan', {
            rows: formatRows(fullScan[1], language),
            relation: fullScan[2],
        });
    }

    const scanAndSort = summary.match(SCAN_AND_SORT);
    if (scanAndSort) {
        return t('chat.toolDenial.explainScanAndSort', {
            rows: formatRows(scanAndSort[1], language),
        });
    }

    if (summary === 'AI query blocked because EXPLAIN detected a Cartesian join.') {
        return t('chat.toolDenial.explainCartesianJoin');
    }

    if (summary === 'AI query blocked because the EXPLAIN output could not be evaluated safely.') {
        return t('chat.toolDenial.explainUnsafeOutput');
    }

    return summary;
}
