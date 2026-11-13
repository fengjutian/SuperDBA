import {useCallback, useEffect, useId, useState} from 'react';
import {Check, Copy, Loader2, Maximize2} from 'lucide-react';
import {useTranslation} from 'react-i18next';
import {MermaidFullscreenModal} from '@/components/modals/MermaidFullscreenModal';
import {useCopyFeedback} from '@/hooks/useCopyFeedback';
import {useTheme} from '@/hooks/useTheme';
import {normalizeMermaidSource} from '@/utils/mermaid';

interface RenderedDiagram {
    source: string;
    themeId: string;
    svg?: string;
    failed?: boolean;
}

export function MermaidDiagram({source}: {source: string}) {
    const {t} = useTranslation();
    const diagramId = `chat-mermaid-${useId().replace(/[^a-zA-Z0-9_-]/g, '')}`;
    const {currentTheme} = useTheme();
    const {copied, copy} = useCopyFeedback(1500);
    const [rendered, setRendered] = useState<RenderedDiagram | null>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);
    const isCurrent = rendered?.source === source && rendered.themeId === currentTheme.id;
    const renderSource = normalizeMermaidSource(source);
    const closeFullscreen = useCallback(() => setIsFullscreen(false), []);

    useEffect(() => {
        let cancelled = false;

        async function renderDiagram() {
            try {
                const {default: mermaid} = await import('mermaid');
                const {colors, typography} = currentTheme;
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
    }, [currentTheme, diagramId, renderSource, source]);

    const copyToolbar = (
        <div className="flex justify-end gap-1 border-b border-default bg-surface-secondary px-2 py-1">
            <button
                type="button"
                onClick={() => setIsFullscreen(true)}
                disabled={!isCurrent || !rendered?.svg}
                aria-label={t('chat.mermaidFullscreen', {defaultValue: 'Fullscreen'})}
                className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-accent hover:bg-surface-hover hover:text-primary disabled:cursor-not-allowed disabled:text-disabled focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
            >
                <Maximize2 size={12} />
                {t('chat.mermaidFullscreen', {defaultValue: 'Fullscreen'})}
            </button>
            <button
                type="button"
                onClick={() => void copy(renderSource)}
                aria-label={t('common.copy')}
                className="inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[11px] text-accent hover:bg-surface-hover hover:text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
            >
                {copied ? <Check size={12} /> : <Copy size={12} />}
                {copied ? t('chat.copied', {defaultValue: 'Copied'}) : t('common.copy')}
            </button>
        </div>
    );

    if (!isCurrent) {
        return (
            <div className="my-2 overflow-hidden rounded border border-default">
                {copyToolbar}
                <div className="flex min-h-24 items-center justify-center bg-surface-secondary">
                    <Loader2 className="size-5 animate-spin text-muted" aria-label="Mermaid" />
                </div>
            </div>
        );
    }

    if (rendered.failed || !rendered.svg) {
        return (
            <div className="my-2 overflow-hidden rounded border border-default">
                {copyToolbar}
                <pre className="overflow-x-auto bg-surface-secondary p-2">
                    <code className="language-mermaid">{renderSource}</code>
                </pre>
            </div>
        );
    }

    return (
        <>
            <div className="my-2 max-w-full overflow-hidden rounded border border-default bg-base">
                {copyToolbar}
                <div
                    role="img"
                    aria-label="Mermaid"
                    className="overflow-x-auto p-3 [&_svg]:mx-auto [&_svg]:h-auto [&_svg]:max-w-none"
                    dangerouslySetInnerHTML={{__html: rendered.svg}}
                />
            </div>
            <MermaidFullscreenModal
                isOpen={isFullscreen}
                onClose={closeFullscreen}
                source={renderSource}
                svg={rendered.svg}
            />
        </>
    );
}
