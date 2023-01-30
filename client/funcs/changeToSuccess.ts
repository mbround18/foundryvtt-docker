import http from "axios";

function getClasses(ele: HTMLElement) {
  return (ele.getAttribute("class") || "").split(" ");
}

function classesMinusHidden(ele: HTMLElement) {
  return getClasses(ele).filter((e) => e != "hidden");
}

function removeHidden(ele: HTMLElement) {
  ele.setAttribute("class", classesMinusHidden(ele).join(" "));
}

function addHidden(ele: HTMLElement) {
  ele.setAttribute("class", [...classesMinusHidden(ele), "hidden"].join(" "));
}

export default () => {
  const instructions = document.getElementById("instruction-container");
  const success = document.getElementById("success-content");
  addHidden(instructions);
  removeHidden(success);
  const interval = setInterval(async () => {
    const { status } = await http.get("/license");
    if (status === 200) {
      clearInterval(interval);
      window.location.replace("/license");
    }
  }, 1000);
};
