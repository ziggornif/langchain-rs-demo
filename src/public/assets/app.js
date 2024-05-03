import markdownIt from 'https://cdn.jsdelivr.net/npm/markdown-it@14.1.0/+esm';

async function postPrompt(promptMessage) {
  const resp = await fetch("/prompt", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      question: promptMessage,
    })
  })

  if (!resp.ok) {
    throw new Error('Network response was not ok');
  }

  return resp.body.getReader();
}

function addLoader(element) {
  const loader = document.createElement('span');
  loader.setAttribute("aria-busy", "true")
  loader.id = "loader";
  loader.textContent = "🤖 is typing...";
  element.appendChild(loader)
}

function removeLoader(element) {
  const child = document.getElementById("loader");
  element.removeChild(child);
}

window.onload = async function () {
  const promptForm = document.getElementById("prompt-form");

  promptForm.addEventListener('submit', async (event) => {
    event.preventDefault();
    const promptElem = document.getElementById("prompt");
    const promptMessage = promptElem.value;
    const messagesDiv = document.getElementById("messages");
    const authorPrompt = document.createElement('p');
    const robotPrompt = document.createElement('p');

    const userAvatar = document.createElement('img');
    userAvatar.src = 'assets/img/you_avatar.png';
    userAvatar.alt = 'user avatar';
    authorPrompt.appendChild(userAvatar)
    const userPrompt = document.createTextNode(` ${promptMessage}`);
    authorPrompt.appendChild(userPrompt);
    messagesDiv.appendChild(authorPrompt);
    addLoader(messagesDiv);
    promptElem.value = null;
    promptElem.disabled = true;
    const reader = await postPrompt(promptMessage);
    removeLoader(messagesDiv);
    robotPrompt.innerHTML = "";
    messagesDiv.appendChild(robotPrompt);
    const decoder = new TextDecoder('utf-8');  
    // Read chunks of data from the stream
    let text = "![avatar](assets/img/avatar.png) ";
    const readStream = () => {
      reader.read().then(({ done, value }) => {
        if (done) {
            return;
        }
        const chunkString = decoder.decode(value);
        text += chunkString;
        robotPrompt.innerHTML = markdownIt().render(text);
        promptElem.scrollIntoView();
        readStream();
      }).catch(error => {
          console.error('Error reading stream:', error);
      });
    };
    readStream();
    promptElem.disabled = false;
    messagesDiv.appendChild(document.createElement("hr"));
  })
}