import { IncomingMessage, ServerResponse } from 'http'
import { useBody, send, appendHeader } from 'h3'
import { downloadTimedUrl } from "../utils/downloadTimedUrl.mjs";
import { FOUNDRY_VTT_ZIP_PATH } from "../utils/dirs.mjs"
import json from "../utils/json.mjs";

const timedLinkRegex = /https\W+foundryvtt.+releases.+AWSAccessKeyId.+Signature.+Expires.+/;

/**
 *
 * @param {IncomingMessage} req
 * @param {ServerResponse} res
 * @returns {Promise<void>}
 */
export default async (req, res) => {
    const body = await useBody(req);
    const timedLink = body["foundry"];
    console.log('Timed link received : ', { timedLink })

    await json(res, {
        message: 'success'
    })
    console.log("Post Processing");
    if (!timedLinkRegex.test(timedLink)) {
        console.log({
          message: "The link your sent does not match expected!"
        });
    }
    console.log('Timed link is valid : ', { timedLink, valid: true });
    if (timedLink) {
        console.log('Downloading Timed link : ', { timedLink, valid: true, status: 'downloading' })
        await downloadTimedUrl({
            url: timedLink,
            destinationPath: FOUNDRY_VTT_ZIP_PATH,
        });
        console.log('Downloading Timed link : ', { timedLink, valid: true, status: 'downloaded' })
        console.log({ status: 'success' });
        process.exit(0);
    } else {
        console.log('Posted Information: ', { timedLink })
        console.log({message: `Something went wrong!`});
    }
}
