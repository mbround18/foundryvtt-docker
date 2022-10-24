import {appendHeader, send} from "h3";

export default async (res, data) => {
    await appendHeader(res, 'Content-Type', 'application/json');
    await send(res,  typeof data === 'string' ? data : JSON.stringify(data))
}
