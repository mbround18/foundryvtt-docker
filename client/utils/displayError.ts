export default (hide: boolean = true, message: string = "") => {
  const errorDiv = document.getElementById("url-error");
  if (hide) {
    errorDiv.setAttribute("class", "hidden");
  } else {
    errorDiv.setAttribute("class", "visible text-red-600");
  }
  errorDiv.innerText = message;
};
