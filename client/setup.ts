import theming from "~/client/utils/theming";
import { get } from "lodash";
import displayError from "~/client/utils/displayError";
import agent from "axios";
import changeToSuccess from "~/client/funcs/changeToSuccess";

export default () => {
  theming();
  const urlInputField = document.getElementById("foundry-url");
  const submitBtn = document.getElementById("foundry-url-submit");
  let suppliedUrl = "";

  urlInputField.onchange = ({ target }) => {
    suppliedUrl = get(target, "value", "");
  };

  submitBtn.onclick = async () => {
    const source = suppliedUrl || get(urlInputField, "value");
    if (!source.startsWith("https://foundryvtt.s3.amazonaws.com")) {
      displayError(
        false,
        "Url did not match a known FoundryVTT download schema. \nPlease recopy the timed url and paste it in the field above."
      );
    } else {
      const res = await agent.post("/uploader", {
        foundry: source,
      });
      const { status } = res;
      if (status === 200) {
        changeToSuccess();
      } else {
        console.error(res);
      }
    }
  };
};
