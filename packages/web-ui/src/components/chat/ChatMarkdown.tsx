import Markdown, {type Components} from 'react-markdown';
import remarkGfm from 'remark-gfm';

/**
 * GitHub-flavoured Markdown (tables, strikethrough, task lists, autolinks) is
 * what assistant models actually emit, so the chat transcript opts into it
 * instead of Core CommonMark. Everything below styles the generated HTML with
 * theme tokens; see `.rules/design.md`.
 */
const STYLES = [
    'max-w-full break-words text-sm leading-relaxed text-secondary',
    // Paragraphs and last-child spacing
    '[&_p]:mb-2 [&_p]:leading-relaxed [&_p:last-child]:mb-0',
    // Headings stay compact: the side panel is narrow.
    '[&_h1]:mt-4 [&_h1]:mb-2 [&_h1]:text-base [&_h1]:font-semibold [&_h1]:text-primary',
    '[&_h2]:mt-3 [&_h2]:mb-1.5 [&_h2]:text-sm [&_h2]:font-semibold [&_h2]:text-primary',
    '[&_h3]:mt-3 [&_h3]:mb-1 [&_h3]:text-sm [&_h3]:font-medium [&_h3]:text-primary',
    '[&_h4]:mt-2 [&_h4]:mb-1 [&_h4]:text-xs [&_h4]:font-semibold [&_h4]:uppercase [&_h4]:tracking-wide [&_h4]:text-muted',
    // Lists
    '[&_ul]:mb-2 [&_ul]:list-disc [&_ul]:pl-5',
    '[&_ol]:mb-2 [&_ol]:list-decimal [&_ol]:pl-5',
    '[&_li]:mb-0.5 [&_li]:pl-0.5 [&_li>p]:mb-0 [&_li>ul]:mt-1 [&_li>ol]:mt-1',
    // Code
    '[&_code]:rounded [&_code]:bg-surface-secondary [&_code]:px-1 [&_code]:py-0.5 [&_code]:font-mono [&_code]:text-xs [&_code]:text-primary',
    '[&_pre]:my-2 [&_pre]:overflow-x-auto [&_pre]:rounded [&_pre]:border [&_pre]:border-default [&_pre]:bg-surface-secondary [&_pre]:p-2',
    '[&_pre_code]:bg-transparent [&_pre_code]:p-0 [&_pre_code]:text-secondary',
    // Tables (the scroll wrapper comes from the `table` renderer below)
    '[&_table]:w-full [&_table]:border-collapse [&_table]:text-xs',
    '[&_th]:border-b [&_th]:border-strong [&_th]:bg-surface-secondary [&_th]:px-2 [&_th]:py-1 [&_th]:text-left [&_th]:font-semibold [&_th]:text-primary',
    '[&_td]:border-b [&_td]:border-default [&_td]:px-2 [&_td]:py-1 [&_td]:align-top',
    // Blockquotes, rules, emphasis, media
    '[&_blockquote]:my-2 [&_blockquote]:border-l-2 [&_blockquote]:border-strong [&_blockquote]:pl-2 [&_blockquote]:text-muted',
    '[&_hr]:my-3 [&_hr]:border-default',
    '[&_strong]:font-semibold [&_strong]:text-primary',
    '[&_em]:italic',
    '[&_del]:text-muted [&_del]:line-through',
    '[&_img]:max-w-full [&_img]:rounded',
    // Task lists
    '[&_input[type=checkbox]]:mr-1 [&_input[type=checkbox]]:align-middle',
].join(' ');

const COMPONENTS: Components = {
    // A wide table must scroll inside the panel instead of widening the transcript.
    table: ({children}) => (
        <div className="my-2 overflow-x-auto rounded-md border border-default">
            <table>{children}</table>
        </div>
    ),
    a: ({href, children}) => (
        <a
            href={href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent underline underline-offset-2 hover:text-primary"
        >
            {children}
        </a>
    ),
};

export function ChatMarkdown({content}: {content: string}) {
    return (
        <div className={STYLES}>
            <Markdown remarkPlugins={[remarkGfm]} components={COMPONENTS}>
                {content}
            </Markdown>
        </div>
    );
}
