import type { IncomingMessage, ServerResponse } from 'http'
export default async function f(req: IncomingMessage, res: ServerResponse) {
    res.statusCode = 200
    res.end('success')
    setTimeout(() => {
        process.exit(0)
    }, 2000)
}