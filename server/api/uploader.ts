import {Request, Response} from 'express';
import { downloadTimedUrl } from "../utils/downloadTimedUrl";
import { FOUNDRY_VTT_ZIP_PATH } from "../utils/dirs"

const timedLinkRegex = /https\W+foundryvtt.+releases.+AWSAccessKeyId.+Signature.+Expires.+/;

export default async (req: Request, response: Response) => {
    console.log(req)
    const body = req.body
    const timedLink = body["foundry"];
    console.log('Timed link received : ', { timedLink })
    response.status(200).send({
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
