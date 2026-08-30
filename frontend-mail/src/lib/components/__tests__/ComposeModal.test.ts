import { describe, it, expect } from 'vitest';

describe('ComposeModal Logic Tests', () => {
    function removeSignature(body: string, activeSignatureId: string | null, lastInjectedSignatureContent: string) {
        if (!activeSignatureId) return { newBody: body, activeSignatureId: null, lastInjectedSignatureContent: '' };

        const sigStart = '\n\n<!-- data-kestrel-signature: start -->\n';
        const sigEnd = '\n<!-- data-kestrel-signature: end -->';

        const startIndex = body.indexOf(sigStart);
        const endIndex = body.indexOf(sigEnd);

        if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
            const currentSignatureContent = body.substring(startIndex + sigStart.length, endIndex);

            if (currentSignatureContent !== lastInjectedSignatureContent) {
                // simulate confirm returning true
                return {
                    newBody: body.substring(0, startIndex) + body.substring(endIndex + sigEnd.length),
                    activeSignatureId: null,
                    lastInjectedSignatureContent: ''
                };
            }

            return {
                newBody: body.substring(0, startIndex) + body.substring(endIndex + sigEnd.length),
                activeSignatureId: null,
                lastInjectedSignatureContent: ''
            };
        }

        return {
            newBody: body,
            activeSignatureId: null,
            lastInjectedSignatureContent: ''
        };
    }

    function swapSignature(body: string, activeSignatureId: string | null, lastInjectedSignatureContent: string, newSigId: string, newSigContent: string) {
        const sigStart = '\n\n<!-- data-kestrel-signature: start -->\n';
        const sigEnd = '\n<!-- data-kestrel-signature: end -->';

        if (activeSignatureId) {
            const startIndex = body.indexOf(sigStart);
            const endIndex = body.indexOf(sigEnd);

            if (startIndex !== -1 && endIndex !== -1 && endIndex > startIndex) {
                return {
                    newBody: body.substring(0, startIndex) + sigStart + newSigContent + sigEnd + body.substring(endIndex + sigEnd.length),
                    activeSignatureId: newSigId,
                    lastInjectedSignatureContent: newSigContent
                };
            }
        }

        return {
            newBody: body + sigStart + newSigContent + sigEnd,
            activeSignatureId: newSigId,
            lastInjectedSignatureContent: newSigContent
        };
    }

    it('swaps signature correctly when changing accounts', () => {
        let body = 'My email body';
        let activeSignatureId: string | null = null;
        let lastContent = '';

        // Initial inject
        const res1 = swapSignature(body, activeSignatureId, lastContent, 'sig1', '<b>Work Sig</b>');
        body = res1.newBody;
        activeSignatureId = res1.activeSignatureId;
        lastContent = res1.lastInjectedSignatureContent;

        expect(body).toBe('My email body\n\n<!-- data-kestrel-signature: start -->\n<b>Work Sig</b>\n<!-- data-kestrel-signature: end -->');

        // Swap
        const res2 = swapSignature(body, activeSignatureId, lastContent, 'sig2', '<i>Personal Sig</i>');
        body = res2.newBody;
        activeSignatureId = res2.activeSignatureId;
        lastContent = res2.lastInjectedSignatureContent;

        expect(body).toBe('My email body\n\n<!-- data-kestrel-signature: start -->\n<i>Personal Sig</i>\n<!-- data-kestrel-signature: end -->');
    });

    it('removes signature completely via remove option', () => {
        let body = 'My email body\n\n<!-- data-kestrel-signature: start -->\n<b>Work Sig</b>\n<!-- data-kestrel-signature: end -->';
        const res = removeSignature(body, 'sig1', '<b>Work Sig</b>');
        expect(res.newBody).toBe('My email body');
    });
});
