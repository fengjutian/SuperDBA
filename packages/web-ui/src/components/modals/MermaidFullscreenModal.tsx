import {Check, Copy, Maximize2, X} from 'lucide-react';
import {useId} from 'react';
import {useTranslation} from 'react-i18next';
import {useCopyFeedback} from '@/hooks/useCopyFeedback';
import {Modal} from '@/components/ui/Modal';

interface MermaidFullscreenModalProps {
    isOpen: boolean;
    onClose: () => void;
    source: string;
    svg: string;
}

export function MermaidFullscreenModal({
    isOpen,
    onClose,
    source,
    svg,
}: MermaidFullscreenModalProps) {
    const {t} = useTranslation();
    const titleId = useId();
    const {copied, copy} = useCopyFeedback(1500);

    return (
        <Modal isOpen={isOpen} onClose={onClose} closeOnBackdrop>
            <section
                role="dialog"
                aria-modal="true"
                aria-labelledby={titleId}
                className="flex h-[calc(100vh-2rem)] w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-xl border border-strong bg-elevated shadow-2xl"
            >
                <header className="flex items-center justify-between border-b border-default bg-base px-4 py-3">
                    <div className="flex items-center gap-2">
                        <Maximize2 size={18} className="text-accent" />
                        <h2 id={titleId} className="text-sm font-semibold text-primary">
                            {t('chat.mermaidTitle', {defaultValue: 'Mermaid diagram'})}
                        </h2>
                    </div>
                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={() => void copy(source)}
                            aria-label={t('common.copy')}
                            className="inline-flex items-center gap-1 rounded px-2 py-1 text-xs text-accent hover:bg-surface-hover hover:text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                        >
                            {copied ? <Check size={14} /> : <Copy size={14} />}
                            {copied ? t('chat.copied', {defaultValue: 'Copied'}) : t('common.copy')}
                        </button>
                        <button
                            type="button"
                            onClick={onClose}
                            aria-label={t('common.close')}
                            autoFocus
                            className="rounded p-1.5 text-secondary hover:bg-surface-hover hover:text-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus"
                        >
                            <X size={18} />
                        </button>
                    </div>
                </header>
                <div className="min-h-0 flex-1 overflow-auto bg-base p-4">
                    <div
                        role="img"
                        aria-label="Mermaid"
                        className="min-h-full min-w-full [&_svg]:mx-auto [&_svg]:h-auto [&_svg]:max-w-none"
                        dangerouslySetInnerHTML={{__html: svg}}
                    />
                </div>
            </section>
        </Modal>
    );
}
