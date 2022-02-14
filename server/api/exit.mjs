import { IncomingMessage, ServerResponse } from 'http'

/**
 *
 * @param {IncomingMessage} req
 * @param {ServerResponse} res
 * @returns {Promise<void>}
 */
export default async function f(req, res) {
    res.statusCode = 200
    res.end('success')
    setTimeout(() => {
        process.exit(0)
    }, 2000)
}
