import { useEffect, useState } from "react";
import { listen } from '@tauri-apps/api/event';
import "./App.css";

type ActiveWindow = {
  title: string;
  appName: string;
};


function App() {
  const [activeWindow, setActiveWindow] = useState<ActiveWindow>({
    title: "",
    appName: "",
  });
  const [message, setMessage] = useState("");

  useEffect(() => {
    listen<ActiveWindow>('active-window-changed', (event) => {
      console.log(event);
      setActiveWindow(event.payload);
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
      <div>
        <h2>{activeWindow.appName}</h2>
        <p>{activeWindow.title}</p>
      </div>
      <div>
        <p>Message: {message}</p>
      </div>
    </main>
  );
}

export default App;
