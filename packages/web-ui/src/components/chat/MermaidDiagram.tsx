import {useEffect, useId, useState} from 'react';
import {Loader2} from 'lucide-react';
import {useTheme} from '@/hooks/useTheme';
import {normalizeMermaidSource} from '@/utils/mermaid';

interface RenderedDiagram {
    source: string;
    themeId: string;
    svg?: string;
    failed?: boolean;
}

export function MermaidDiagram({source}: {source: string}) {
    const diagramId = `chat-mermaid-${useId().replace(/[^a-zA-Z0-9_-]/g, '')}`;
    const {currentTheme} = useTheme();
    const [rendered, setRendered] = useState<RenderedDiagram | null>(null);
    const isCurrent = rendered?.source === source && rendered.themeId === currentTheme.id;

    useEffect(() => {
        let cancelled = false;

        async function renderDiagram() {
            try {
                const {default: mermaid} = await import('mermaid');
                const {colors, typography} = currentTheme;
                const renderSource = normalizeMermaidSource(source);
                mermaid.initialize({
                    startOnLoad: false,
                    securityLevel: 'strict',
                    suppressErrorRendering: true,
                    theme: 'base',
                    themeVariables: {
                        background: colors.bg.base,
                        primaryColor: colors.surface.secondary,
                        primaryTextColor: colors.text.primary,
                        primaryBorderColor: colors.border.strong,
                        secondaryColor: colors.surface.tertiary,
                        tertiaryColor: colors.bg.elevated,
                        lineColor: colors.text.muted,
                        edgeLabelBackground: colors.bg.base,
                        clusterBkg: colors.bg.elevated,
                        clusterBorder: colors.border.default,
                        fontFamily: typography.fontFamily.base,
                    },
                    flowchart: {htmlLabels: false, useMaxWidth: true},
                });
                const {svg} = await mermaid.render(diagramId, renderSource);
                if (!cancelled) {
                    setRendered({source, themeId: currentTheme.id, svg});
                }
            } catch {
                if (!cancelled) {
                    setRendered({source, themeId: currentTheme.id, failed: true});
                }
            }
        }

        void renderDiagram();
        return () => {
            cancelled = true;
        };
    }, [currentTheme, diagramId, source]);

    if (!isCurrent) {
        return (
            <div className="my-2 flex min-h-24 items-center justify-center rounded border border-default bg-surface-secondary">
                <Loader2 className="size-5 animate-spin text-muted" aria-label="Mermaid" />
            </div>
        );
    }

    if (rendered.failed || !rendered.svg) {
        return (
            <pre className="my-2 overflow-x-auto rounded border border-default bg-surface-secondary p-2">
                <code className="language-mermaid">{source}</code>
            </pre>
        );
    }

    return (
        <div
            role="img"
            aria-label="Mermaid"
            className="my-2 max-w-full overflow-x-auto rounded border border-default bg-base p-3 [&_svg]:mx-auto [&_svg]:h-auto [&_svg]:max-w-none"
            dangerouslySetInnerHTML={{__html: rendered.svg}}
        />
    );
}
