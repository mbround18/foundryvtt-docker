import type { IncomingMessage, ServerResponse } from 'http'
import { useBody, useCookies, useQuery } from 'h3'
import { downloadTimedUrl } from "../utils/downloadTimedUrl";
import { FOUNDRY_VTT_ZIP_PATH } from "../utils/dirs"

const timedLinkRegex = /https\W+foundryvtt.+releases.+AWSAccessKeyId.+Signature.+Expires.+/;

export default async (req: IncomingMessage, res: ServerResponse) => {
    const timedLink = await useBody(req)?.["foundry"];
    console.log('Timed link received : ', { timedLink })
    if (!timedLinkRegex.test(timedLink)) {
        res.statusCode = 400
        return res.end("The link your sent does not match expected!");
    }

    console.log('Timed link is valid : ', { timedLink, valid: true });

    if (timedLink) {
        console.log('Downloading Timed link : ', { timedLink, valid: true, status: 'downloading' })
        await downloadTimedUrl({
            url: timedLink,
            destinationPath: FOUNDRY_VTT_ZIP_PATH,
        });
        console.log('Downloading Timed link : ', { timedLink, valid: true, status: 'downloaded' })
        return res.end({ status: 'success' })
    }
    console.log('Posted Information: ', { timedLink })
    res.statusCode = 500
    return res.end(`Something went wrong!`)
}
