function disableUI(disabled) {
  const elements = document.querySelectorAll('button, input, select, textarea, fieldset');
  elements.forEach(el => el.disabled = disabled);
}

document.getElementById("url-tab").addEventListener("click", () => {
  if (!document.getElementById("url-tab").classList.contains("active")) {
    switchTab("url");
  }
});

document.getElementById("file-tab").addEventListener("click", () => {
  if (!document.getElementById("file-tab").classList.contains("active")) {
    switchTab("file");
  }
});

function switchTab(tabName) {
  // Remove active class from all tabs
  document.querySelectorAll(".tab-button").forEach((tab) => {
    tab.classList.remove("active");
  });

  // Hide all tab contents
  document.querySelectorAll(".method-content").forEach((content) => {
    content.style.display = "none";
  });

  // Set active tab and show content
  document.getElementById(`${tabName}-tab`).classList.add("active");
  document.getElementById(`${tabName}-method`).style.display = "block";
}

// URL download functionality
document
  .getElementById("download-button")
  .addEventListener("click", async () => {
    const url = document.getElementById("url-input").value;
    if (url) {
      // Show waiting symbol and progress container
      showProcessingUI(true);

      try {
        await fetch("/download", {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ url }),
        });
        showToast(
          "Download initiated. The server will exit once the download and extraction are complete.",
          "green",
        );
      } catch (error) {
        console.error("Error initiating download:", error);
        showToast("Failed to initiate download. Please try again.", "red");
        showProcessingUI(false);
      }
    } else {
      showToast("Please enter a URL.", "red");
    }
  });

// File upload functionality
let selectedFile = null;

// File input change handler
document.getElementById("file-input").addEventListener("change", (e) => {
  if (e.target.files.length > 0) {
    selectedFile = e.target.files[0];
    document.getElementById("file-name").textContent = selectedFile.name;
    document.getElementById("upload-button").disabled = false;
  }
});

// File select button
document.getElementById("file-select-button").addEventListener("click", () => {
  document.getElementById("file-input").click();
});

// Upload button click handler
document.getElementById("upload-button").addEventListener("click", async () => {
  if (selectedFile) {
    showProcessingUI(true);

    const formData = new FormData();
    formData.append("file", selectedFile);

    try {
      await fetch("/upload", {
        method: "POST",
        body: formData,
      });
      showToast(
        "Upload initiated. The server will exit once the upload and extraction are complete.",
        "green",
      );
    } catch (error) {
      console.error("Error uploading file:", error);
      showToast("Failed to upload file. Please try again.", "red");
      showProcessingUI(false);
    }
  } else {
    showToast("Please select a file to upload.", "red");
  }
});

// Drag and drop functionality
const dropArea = document.getElementById("drop-area");

["dragenter", "dragover", "dragleave", "drop"].forEach((eventName) => {
  dropArea.addEventListener(eventName, preventDefaults, false);
});

function preventDefaults(e) {
  e.preventDefault();
  e.stopPropagation();
}

["dragenter", "dragover"].forEach((eventName) => {
  dropArea.addEventListener(eventName, highlight, false);
});

["dragleave", "drop"].forEach((eventName) => {
  dropArea.addEventListener(eventName, unhighlight, false);
});

function highlight() {
  dropArea.classList.add("highlight");
}

function unhighlight() {
  dropArea.classList.remove("highlight");
}

dropArea.addEventListener("drop", handleDrop, false);

function handleDrop(e) {
  const dt = e.dataTransfer;
  const files = dt.files;

  if (files.length > 0) {
    selectedFile = files[0];
    document.getElementById("file-name").textContent = selectedFile.name;
    document.getElementById("upload-button").disabled = false;
  }
}

// Helper functions
function showToast(message, color) {
  const toast = document.getElementById("toast");
  toast.textContent = message;
  toast.className = "toast";
  toast.classList.add(color);
  toast.style.display = "block";

  setTimeout(() => {
    toast.style.display = "none";
  }, 5000);
}

function showProcessingUI(processing) {
  // Disable all interactive UI elements
  disableUI(processing);
  
  // Show/hide progress container
  document.getElementById("progress-container").style.display = processing ? "block" : "none";
  
  // Hide/show instruction and tab sections during processing
  document.querySelector(".instructions").style.display = processing ? "none" : "block";
  document.querySelector(".method-tabs").style.display = processing ? "none" : "flex";
  document.querySelectorAll(".method-content").forEach(el => {
    if (processing) {
      el.style.display = "none";
    } else if (el.id === document.querySelector(".tab-button.active").id.replace("-tab", "-method")) {
      el.style.display = "block";
    }
  });
  
  if (processing) {
    connectToEventSource();
  }
}

// SSE Event Source connection
function connectToEventSource() {
  const eventSource = new EventSource("/events");
  const progressBar = document.getElementById("progress-bar");
  const progressMessage = document.getElementById("progress-message");

  eventSource.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data);
      console.log("Event received:", data);

      // Update progress display based on event data
      if (data.progress !== undefined) {
        progressBar.style.width = `${data.progress}%`;
      }

      if (data.message) {
        progressMessage.textContent = data.message;
      }

      // Handle different event types
      if (data.event_type === "downloading") {
        // Update UI for download progress
      } else if (data.event_type === "extracting") {
        // Update UI for extraction progress
      } else if (
        data.type === "complete" ||
        data.type === "error" ||
        data.event_type === "complete" ||
        data.event_type === "error"
      ) {
        eventSource.close();

        if (data.type === "error" || data.event_type === "error") {
          showToast(
            "An error occurred: " + (data.message || "Unknown error"),
            "red",
          );
          showProcessingUI(false);
        } else if (data.type === "complete" || data.event_type === "complete") {
          // Start attempting to redirect to Foundry VTT after 5 seconds
          progressMessage.textContent = "Installation complete! Waiting for server to restart...";
          setTimeout(() => {
            progressMessage.textContent = "Installation complete! Attempting to redirect...";
            attemptRedirect();
          }, 5000);
        }
      }
    } catch (error) {
      console.error("Error processing event:", error);
    }
  };

  eventSource.onerror = () => {
    console.error("EventSource connection error");
    eventSource.close();
    showProcessingUI(false);
  };
}

// Redirect functionality
function attemptRedirect() {
  const progressContainer = document.getElementById("progress-container");
  const progressMessage = document.getElementById("progress-message");
  
  // Create redirect status element if it doesn't exist
  let redirectStatus = document.getElementById("redirect-status");
  if (!redirectStatus) {
    redirectStatus = document.createElement("div");
    redirectStatus.id = "redirect-status";
    redirectStatus.className = "redirect-status";
    progressContainer.appendChild(redirectStatus);
  }
  
  let attempts = 0;
  const maxAttempts = 20; // Try for about 1 minute (20 attempts * 3 seconds)
  
  function tryRedirect() {
    attempts++;
    redirectStatus.textContent = `Redirect attempt ${attempts}/${maxAttempts}...`;
    
    // Try both possible URLs
    Promise.all([
      fetch("/", { method: "HEAD" }).catch(() => ({ status: 404 })),
      fetch("/license", { method: "HEAD" }).catch(() => ({ status: 404 }))
    ]).then(([rootResponse, licenseResponse]) => {
      if (rootResponse.status === 200) {
        // Root URL is available, redirect there
        redirectStatus.textContent = "Foundry VTT is ready! Redirecting...";
        setTimeout(() => window.location.href = "/", 1000);
      } else if (licenseResponse.status === 200) {
        // License page is available, redirect there
        redirectStatus.textContent = "Foundry VTT license page is ready! Redirecting...";
        setTimeout(() => window.location.href = "/license", 1000);
      } else if (attempts < maxAttempts) {
        // Try again after delay
        setTimeout(tryRedirect, 3000);
      } else {
        // Max attempts reached
        redirectStatus.textContent = "Max redirect attempts reached. Please try manually navigating to Foundry VTT.";
        showToast("Foundry VTT might need more time to start. Try refreshing this page in a moment.", "orange");
      }
    });
  }
  
  // Start the redirect attempts
  tryRedirect();
}

