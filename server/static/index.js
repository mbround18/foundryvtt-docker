document
  .getElementById("download-button")
  .addEventListener("click", async () => {
    const url = document.getElementById("url-input").value;
    if (url) {
      // Show waiting symbol
      toggleWaitingSymbol(true);

      try {
        await fetch("/download", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ url }),
        });
        monitorConnection();
        showToast(
          "Download initiated. The server will exit once the download and extraction are complete.",
          "green",
        );
      } catch (error) {
        console.error("Error initiating download:", error);
        showToast("Failed to initiate download. Please try again.", "red");
      }
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

function toggleWaitingSymbol(show) {
  document.getElementById("waiting-symbol").style.display = show
    ? "block"
    : "none";
}

async function monitorConnection() {
  const interval = 5000; // 5 seconds
  while (true) {
    try {
      const response = await fetch("/license");
      if (response.ok) {
        window.location.href = "/";
        break;
      }
    } catch (error) {
      console.warn("Connection check failed. Retrying...");
    }
    await new Promise((resolve) => setTimeout(resolve, interval));
  }
}
