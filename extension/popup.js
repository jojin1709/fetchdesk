document.addEventListener("DOMContentLoaded", () => {
  const statusDiv = document.getElementById("status");
  const sendBtn = document.getElementById("send");
  const urlInput = document.getElementById("url");

  // Ping the local webhook host (via OPTIONS) to check status
  fetch("http://127.0.0.1:8382/download", { method: "OPTIONS" })
    .then(() => {
      statusDiv.textContent = "Connected to FetchDesk app";
      statusDiv.className = "status-ok";
    })
    .catch(() => {
      statusDiv.textContent = "FetchDesk app is not running";
      statusDiv.className = "status-fail";
    });

  sendBtn.addEventListener("click", () => {
    const url = urlInput.value.trim();
    if (!url) return;

    fetch("http://127.0.0.1:8382/download", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ url: url })
    })
    .then(r => {
      if (!r.ok) throw new Error("Server error");
      return r.json();
    })
    .then(data => {
      statusDiv.textContent = `Added to queue (ID: ${data.id})`;
      statusDiv.className = "status-ok";
      urlInput.value = "";
    })
    .catch(() => {
      statusDiv.textContent = "Failed to send link to FetchDesk";
      statusDiv.className = "status-fail";
    });
  });
});
