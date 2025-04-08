// Improved version of index.js - Scoped in an IIFE to avoid global namespace pollution.
(() => {
  "use strict";

  // Track global state
  const state = {
    eventSourceConnected: false,
    processingComplete: false,
    serverShuttingDown: false,
  };

  /**
   * Disables or enables UI elements.
   * @param {boolean} disabled - Whether to disable the UI.
   */
  const disableUI = (disabled) => {
    const elements = document.querySelectorAll(
      "button, input, select, textarea, fieldset",
    );
    elements.forEach((el) => (el.disabled = disabled));
  };

  /**
   * Initializes tab switching functionality.
   */
  const initTabs = () => {
    const tabs = ["url", "file"];
    tabs.forEach((tab) => {
      const tabElement = document.getElementById(`${tab}-tab`);
      if (tabElement) {
        tabElement.addEventListener("click", () => {
          if (!tabElement.classList.contains("active")) {
            switchTab(tab);
          }
        });
      }
    });
  };

  /**
   * Switches active tab and displays corresponding content.
   * @param {string} tabName - The name of the tab to activate.
   */
  const switchTab = (tabName) => {
    document
      .querySelectorAll(".tab-button")
      .forEach((tab) => tab.classList.remove("active"));
    document
      .querySelectorAll(".method-content")
      .forEach((content) => (content.style.display = "none"));

    const activeTab = document.getElementById(`${tabName}-tab`);
    const activeContent = document.getElementById(`${tabName}-method`);
    if (activeTab && activeContent) {
      activeTab.classList.add("active");
      activeContent.style.display = "block";
    }
  };

  /**
   * Processes the given request endpoint with the provided data.
   * @param {string} endpoint - The endpoint URL.
   * @param {FormData|object} data - The data to be sent.
   */
  const processRequest = async (endpoint, data) => {
    showProcessingUI(true);
    try {
      const options = {
        method: "POST",
        ...(data instanceof FormData
          ? { body: data }
          : {
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify(data),
            }),
      };
      const response = await fetch(endpoint, options);
      if (!response.ok) {
        throw new Error(`Server responded with status ${response.status}`);
      }
      showToast(
        "Process initiated. The server will exit once the operation is complete.",
        "green",
      );
    } catch (error) {
      console.error(`Error with ${endpoint}:`, error);
      showToast("Failed to process request. Please try again.", "red");
      showProcessingUI(false);
    }
  };

  /**
   * Initializes URL download functionality.
   */
  const initUrlDownload = () => {
    const downloadButton = document.getElementById("download-button");
    if (downloadButton) {
      downloadButton.addEventListener("click", async () => {
        const urlInput = document.getElementById("url-input");
        const url = urlInput ? urlInput.value : "";
        if (url) {
          await processRequest("/download", { url });
        } else {
          showToast("Please enter a URL.", "red");
        }
      });
    }
  };

  /**
   * Initializes file upload functionality with drag and drop support.
   */
  const initFileUpload = () => {
    let selectedFile = null;
    const fileInput = document.getElementById("file-input");
    const fileNameEl = document.getElementById("file-name");
    const uploadButton = document.getElementById("upload-button");
    const dropArea = document.getElementById("drop-area");

    if (!fileInput || !fileNameEl || !uploadButton || !dropArea) {
      console.warn("Some file upload elements are missing in the DOM.");
      return;
    }

    // Update file selection display
    const updateFileSelection = (file) => {
      fileNameEl.textContent = file.name;
      uploadButton.disabled = false;
    };

    fileInput.addEventListener("change", (e) => {
      if (e.target.files && e.target.files.length > 0) {
        selectedFile = e.target.files[0];
        updateFileSelection(selectedFile);
      }
    });

    const fileSelectButton = document.getElementById("file-select-button");
    if (fileSelectButton) {
      fileSelectButton.addEventListener("click", () => fileInput.click());
    }

    uploadButton.addEventListener("click", async () => {
      if (selectedFile) {
        const formData = new FormData();
        formData.append("file", selectedFile);
        await processRequest("/upload", formData);
      } else {
        showToast("Please select a file to upload.", "red");
      }
    });

    // Initialize drag and drop functionality
    initDragAndDrop(dropArea, (files) => {
      if (files && files.length > 0) {
        selectedFile = files[0];
        updateFileSelection(selectedFile);
      }
    });
  };

  /**
   * Initializes drag and drop events on a given drop area.
   * @param {HTMLElement} dropArea - The drop area element.
   * @param {function(FileList): void} onDropCallback - Callback for handling dropped files.
   */
  const initDragAndDrop = (dropArea, onDropCallback) => {
    const preventDefaults = (e) => {
      e.preventDefault();
      e.stopPropagation();
    };

    const highlight = () => dropArea.classList.add("highlight");
    const unhighlight = () => dropArea.classList.remove("highlight");

    const events = {
      dragenter: preventDefaults,
      dragover: (e) => {
        preventDefaults(e);
        highlight();
      },
      dragleave: (e) => {
        preventDefaults(e);
        unhighlight();
      },
      drop: (e) => {
        preventDefaults(e);
        unhighlight();
        onDropCallback(e.dataTransfer.files);
      },
    };

    Object.entries(events).forEach(([event, handler]) => {
      dropArea.addEventListener(event, handler, false);
    });
  };

  /**
   * Displays a toast message with a given color.
   * @param {string} message - The message to display.
   * @param {string} color - The toast color (e.g., "green", "red").
   */
  const showToast = (message, color) => {
    const toast = document.getElementById("toast");
    if (toast) {
      toast.textContent = message;
      toast.className = "toast";
      toast.classList.add(color);
      toast.style.display = "block";
      setTimeout(() => {
        toast.style.display = "none";
      }, 5000);
    }
  };

  /**
   * Updates the UI to show processing status.
   * @param {boolean} processing - Whether processing is active.
   */
  const showProcessingUI = (processing) => {
    disableUI(processing);
    const progressContainer = document.getElementById("progress-container");
    if (progressContainer) {
      progressContainer.style.display = processing ? "block" : "none";
    }
    const instructions = document.querySelector(".instructions");
    const methodTabs = document.querySelector(".method-tabs");
    if (instructions)
      instructions.style.display = processing ? "none" : "block";
    if (methodTabs) methodTabs.style.display = processing ? "none" : "flex";

    document.querySelectorAll(".method-content").forEach((el) => {
      if (processing) {
        el.style.display = "none";
      } else {
        // Ensure only the active tab's content is visible
        const activeTab = document.querySelector(".tab-button.active");
        if (activeTab && el.id === activeTab.id.replace("-tab", "-method")) {
          el.style.display = "block";
        }
      }
    });

    if (processing && !state.eventSourceConnected) {
      connectToEventSource();
    }
  };

  /**
   * Establishes connection to the server via Server-Sent Events (SSE) for progress updates.
   */
  const connectToEventSource = () => {
    // Only connect once
    if (state.eventSourceConnected) return;

    state.eventSourceConnected = true;
    const eventSource = new EventSource("/events");
    const progressBar = document.getElementById("progress-bar");
    const progressMessage = document.getElementById("progress-message");

    eventSource.onopen = () => {
      console.log("EventSource connection established");
    };

    eventSource.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log("Event received:", data);

        if (data.progress !== undefined && progressBar) {
          progressBar.style.width = `${data.progress}%`;
        }
        if (data.message && progressMessage) {
          progressMessage.textContent = data.message;
        }

        const isComplete =
          data.type === "complete" || data.event_type === "complete";
        const isError = data.type === "error" || data.event_type === "error";

        if (isComplete || isError) {
          console.log(
            `Closing EventSource connection: ${isComplete ? "complete" : "error"} event received`,
          );
          state.eventSourceConnected = false;
          eventSource.close();

          if (isError) {
            showToast(
              "An error occurred: " + (data.message || "Unknown error"),
              "red",
            );
            showProcessingUI(false);
          } else if (isComplete) {
            state.processingComplete = true;
            if (progressMessage) {
              progressMessage.textContent =
                "Installation complete! Waiting 30 seconds for server to shut down...";
            }
            // Hard 20-second wait after complete event
            setTimeout(waitForServerShutdown, 30000);
          }
        }
      } catch (error) {
        console.error("Error processing event:", error, event.data);
        state.eventSourceConnected = false;
        eventSource.close();
        showToast("Error processing server event. Please try again.", "red");
        showProcessingUI(false);
      }
    };

    eventSource.onerror = (error) => {
      console.error("EventSource connection error", error);

      // If we were expecting this due to server shutdown, don't treat as error
      if (state.processingComplete && !state.serverShuttingDown) {
        state.serverShuttingDown = true;
        console.log(
          "EventSource disconnected after completion - likely server shutdown",
        );
        setTimeout(waitForServerShutdown, 10000);
      } else if (!state.processingComplete) {
        showToast("Lost connection to server. Please try again.", "red");
        showProcessingUI(false);
      }

      state.eventSourceConnected = false;
      eventSource.close();
    };

    // Set a timeout to handle cases where the server never responds
    setTimeout(() => {
      if (state.eventSourceConnected && !state.processingComplete) {
        console.warn("EventSource timeout - no activity detected");
        state.eventSourceConnected = false;
        eventSource.close();
        showToast("Server did not respond in time. Please try again.", "red");
        showProcessingUI(false);
      }
    }, 60000); // 1 minute timeout
  };

  /**
   * Attempts to redirect the user to the appropriate URL after process completion.
   */
  const attemptRedirect = () => {
    const progressContainer = document.getElementById("progress-container");
    let redirectStatus = document.getElementById("redirect-status");
    if (!redirectStatus && progressContainer) {
      redirectStatus = document.createElement("div");
      redirectStatus.id = "redirect-status";
      redirectStatus.className = "redirect-status";
      progressContainer.appendChild(redirectStatus);
    }

    let attempts = 0;
    const maxAttempts = 30;
    const delayBetweenAttempts = 3000; // 3 seconds

    const tryRedirect = async () => {
      attempts++;
      if (redirectStatus) {
        redirectStatus.textContent = `Checking for Foundry VTT (${attempts}/${maxAttempts})...`;
      }

      try {
        // Use Promise.allSettled to handle both successful and failed requests
        const [rootResult, licenseResult] = await Promise.allSettled([
          fetch("/", { method: "HEAD" }),
          fetch("/license", { method: "HEAD" }),
        ]);

        const rootResponse =
          rootResult.status === "fulfilled"
            ? rootResult.value
            : { status: 404 };
        const licenseResponse =
          licenseResult.status === "fulfilled"
            ? licenseResult.value
            : { status: 404 };

        if (rootResponse.status === 200) {
          if (redirectStatus) {
            redirectStatus.textContent = "Foundry VTT is ready! Redirecting...";
          }
          setTimeout(() => (window.location.href = "/"), 1000);
        } else if (licenseResponse.status === 200) {
          if (redirectStatus) {
            redirectStatus.textContent =
              "Foundry VTT license page is ready! Redirecting...";
          }
          setTimeout(() => (window.location.href = "/license"), 1000);
        } else if (attempts < maxAttempts) {
          if (redirectStatus) {
            redirectStatus.textContent = `Foundry VTT not ready yet. Next check in ${delayBetweenAttempts / 1000} seconds...`;
          }
          setTimeout(tryRedirect, delayBetweenAttempts);
        } else {
          if (redirectStatus) {
            redirectStatus.textContent =
              "Max redirect attempts reached. Please try manually refreshing the page.";
          }
          showToast(
            "Foundry VTT might need more time to start. Try refreshing this page in a moment.",
            "orange",
          );
        }
      } catch (error) {
        console.warn("Redirect check failed:", error);
        if (attempts < maxAttempts) {
          if (redirectStatus) {
            redirectStatus.textContent = `Connection attempt failed. Retrying in ${delayBetweenAttempts / 1000} seconds...`;
          }
          setTimeout(tryRedirect, delayBetweenAttempts);
        } else {
          if (redirectStatus) {
            redirectStatus.textContent =
              "Unable to connect to Foundry VTT after multiple attempts. Please refresh manually.";
          }
          showToast(
            "Couldn't connect to Foundry VTT. Try refreshing this page in a moment.",
            "red",
          );
        }
      }
    };

    tryRedirect();
  };

  /**
   * Waits for the server to shut down before attempting redirection.
   */
  const waitForServerShutdown = () => {
    if (state.serverShuttingDown) {
      console.log(
        "Already monitoring server shutdown, skipping duplicate call",
      );
      return;
    }

    state.serverShuttingDown = true;
    const progressMessage = document.getElementById("progress-message");
    const progressContainer = document.getElementById("progress-container");

    let shutdownStatus = document.getElementById("shutdown-status");
    if (!shutdownStatus && progressContainer) {
      shutdownStatus = document.createElement("div");
      shutdownStatus.id = "shutdown-status";
      shutdownStatus.className = "redirect-status";
      progressContainer.appendChild(shutdownStatus);
    }

    if (progressMessage) {
      progressMessage.textContent =
        "Server shutdown period complete. Starting redirect attempts...";
    }
    if (shutdownStatus) {
      shutdownStatus.textContent = "Beginning redirect attempts...";
    }

    // Start redirect attempts immediately after the 20-second timeout
    attemptRedirect();
  };

  // Initialize the application once the DOM is ready
  document.addEventListener("DOMContentLoaded", () => {
    initTabs();
    initUrlDownload();
    initFileUpload();
    switchTab("url"); // Set initial tab to URL tab
  });
})();
