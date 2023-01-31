import type { Request, Response } from "express";

export default async function f(req: Request, res: Response) {
  res.status(200).send("success");
  setTimeout(() => {
    process.exit(0);
  }, 2000);
}
