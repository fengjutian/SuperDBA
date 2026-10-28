export type RequestId = string;

export interface SuperDBAError {
    code: string;
    message: string;
    details: unknown | null;
    requestId: RequestId;
}

export class SuperDBAClientError extends Error implements SuperDBAError {
    readonly code: string;
    readonly details: unknown | null;
    readonly requestId: RequestId;

    constructor(error: SuperDBAError, options?: ErrorOptions) {
        super(error.message, options);
        this.name = 'SuperDBAClientError';
        this.code = error.code;
        this.details = error.details;
        this.requestId = error.requestId;
    }
}

export function createRequestId(): RequestId {
    if (typeof globalThis.crypto?.randomUUID === 'function') {
        return globalThis.crypto.randomUUID();
    }

    return `web-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export function normalizeSuperDBAError(
    error: unknown,
    code: string,
    requestId: RequestId = createRequestId(),
): SuperDBAClientError {
    if (error instanceof SuperDBAClientError) return error;

    const message = errorMessage(error);
    return new SuperDBAClientError(
        {
            code,
            message,
            details: null,
            requestId,
        },
        error instanceof Error ? {cause: error} : undefined,
    );
}

function errorMessage(error: unknown): string {
    if (error instanceof Error) return error.message;
    if (typeof error === 'string') return error;
    if (
        typeof error === 'object' &&
        error !== null &&
        'message' in error &&
        typeof error.message === 'string'
    ) {
        return error.message;
    }
    return String(error);
}

export interface RpcSuccess<T> {
    ok: true;
    data: T;
}

export interface RpcFailure {
    ok: false;
    error: SuperDBAError;
}

export type RpcResponse<T> = RpcSuccess<T> | RpcFailure;
