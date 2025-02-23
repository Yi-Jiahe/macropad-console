import { useEffect, useState } from "react";
import { listen } from '@tauri-apps/api/event';
import "./App.css";


function App() {
  const [title, setTitle] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    listen('active-window-changed', (event) => {
      console.log(event);
      const title = event.payload as string;
      console.log(`title: ${title}`);
      setTitle(title);
    });
  }, [])

  useEffect(() => {
    listen('serial-message', (event) => {
      console.log(event);
      const message = event.payload as string;
      console.log(`message: ${message}`);
      setMessage(message);
    });
  }, [])

  return (
    <main className="container">
      <h1>Active Window: {title}</h1>
      <p>Message: {message}</p>
    </main>
  );
}

export default App;
