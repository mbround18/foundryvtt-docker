document
  .getElementById("download-button")
  .addEventListener("click", async () => {
    const url = document.getElementById("url-input").value;
    if (url) {
      // Show waiting symbol
      showWaitingSymbol(true);

      await fetch("/download", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ url }),
      })
        .catch(monitorConnection)
        .then(monitorConnection);

      showToast(
        "Download initiated. The server will exit once the download and extraction are complete.",
        "green",
      );
    } else {
      showToast("Please enter a URL.", "red");
    }
  });

function showToast(message, color) {
  const toast = document.getElementById("toast");
  toast.innerText = message;
  toast.style.backgroundColor = color;
  toast.className = "toast show";
  setTimeout(() => {
    toast.className = toast.className.replace("show", "");
  }, 3000);
}

function showWaitingSymbol(show) {
  const waitingSymbol = document.getElementById("waiting-symbol");
  if (show) {
    waitingSymbol.style.display = "block";
  } else {
    waitingSymbol.style.display = "none";
  }
}

async function monitorConnection() {
  const interval = 5000; // 5 seconds
  while (true) {
    try {
      console.log("Checking connection...");
      const response = await fetch("/license");
      if (response.ok) {
        window.location.href = "/";
        break;
      }
    } catch (error) {
      // Handle connection error
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
}
