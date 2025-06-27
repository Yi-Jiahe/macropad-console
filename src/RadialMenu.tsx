import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

export function RadialMenu() {
  const [visible, setVisible] = useState(false);
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const lastMousePosition = useRef({ x: 0, y: 0 });

  useEffect(() => {
    window.addEventListener("mousemove", (event) => {
      lastMousePosition.current = { x: event.clientX, y: event.clientY };
    });

    listen('show-radial-menu', (event) => {
      setPosition(lastMousePosition.current);
      setVisible(true);
    });

    listen('hide-radial-menu', () => {
      setVisible(false);
    });
  }, []);

  return (
    <>
      {visible && (
        <div
          style={{
            position: 'absolute',
            top: position.y,
            left: position.x,
            transform: 'translate(-50%, -50%)',
            width: '200px',
            height: '200px',
            backgroundColor: 'rgba(0, 0, 0, 0.5)',
            borderRadius: '50%',
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
          }}
        >
          <div className="menu-item">Item 1</div>
          <div className="menu-item">Item 2</div>
          <div className="menu-item">Item 3</div>
          <div className="menu-item">Item 4</div>
          <div className="menu-item">Item 5</div>
        </div>
      )}
    </>
  );
}