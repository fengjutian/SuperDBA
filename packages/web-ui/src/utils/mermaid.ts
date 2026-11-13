const ER_RELATION_WITHOUT_LABEL = /^(\s*\S+\s+[|}{o]+--[|}{o]+\s+\S+)\s*$/;

export function normalizeMermaidSource(source: string): string {
    const lines = source.split('\n');
    const diagramType = lines.find((line) => line.trim().length > 0)?.trim();
    if (diagramType !== 'erDiagram') {
        return source;
    }

    return lines
        .map((line) => {
            if (line.includes(':')) {
                return line;
            }
            const relation = line.match(ER_RELATION_WITHOUT_LABEL);
            return relation ? `${relation[1]} : relates` : line;
        })
        .join('\n');
}
