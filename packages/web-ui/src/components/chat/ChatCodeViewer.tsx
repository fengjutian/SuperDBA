import {CellCodeEditor} from '@/components/ui/CellCodeEditor';

interface ChatCodeViewerProps {
    value: string;
    label: string;
    maxHeight: number;
}

const ignoreReadOnlyChange = () => undefined;

export function ChatCodeViewer({value, label, maxHeight}: ChatCodeViewerProps) {
    const lineCount = value.split('\n').length;
    const height = Math.min(maxHeight, Math.max(72, lineCount * 19 + 16));

    return (
        <div
            role="region"
            aria-label={label}
            className="overflow-hidden rounded border border-default bg-base"
            style={{height}}
        >
            <CellCodeEditor
                value={value}
                onChange={ignoreReadOnlyChange}
                height="100%"
                language="json"
                readOnly
            />
        </div>
    );
}
