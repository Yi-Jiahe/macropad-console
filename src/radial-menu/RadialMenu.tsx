import { useEffect, useRef, useState } from "react";
import { RadialMenuItem, ShowRadialMenuEvent } from "../types";
import { listen } from "@tauri-apps/api/event";
import { radialMenuSize } from "./main";
import { invoke } from "@tauri-apps/api/core";

export function RadialMenu() {
  const [items, setItems] = useState<RadialMenuItem[]>([]);
  const mousePosition = useRef<[number, number] | undefined>();
  const threshold = 50;

  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      mousePosition.current = [e.clientX, e.clientY];
    };
    window.addEventListener('mousemove', handleMouseMove);

    const unlistenShowRadialMenuEvent = listen<ShowRadialMenuEvent>('show-radial-menu', async (event) => {
      setItems(event.payload.items);
    });


    return () => {
      window.removeEventListener('mousemove', handleMouseMove);
      unlistenShowRadialMenuEvent.then(f => f());
    }
  }, []);

  useEffect(() => {
    const unlistenHideRadialMenuEvent = listen('hide-radial-menu', () => {
      if (mousePosition.current === undefined) {
        return;
      }

      let [dx, dy] = mousePosition.current.map(v => v - radialMenuSize / 2);

      let angle = Math.atan2(dy, dx);
      let distance = Math.sqrt(dx * dx + dy * dy);

      console.log("angle", angle, "distance", distance);

      if (distance < threshold) {
        return;
      }

      // Set origin to twelve o'clock
      if (angle < -Math.PI / 2) angle += 2 * Math.PI;
      angle += Math.PI / 2;
      // Rotate origin by section size / 2
      angle += (2 * Math.PI / items.length) / 2;
      angle %= 2 * Math.PI;

      // Find which section
      const section = Math.floor((angle / (2 * Math.PI)) * items.length) % items.length;

      if (isNaN(section)) {
        console.log("Unable to find section");
        return;
      }


      const item = items[section];

      console.log(`Mouse is in section ${section}:`, item?.label);

      invoke('handle_action', { action: item?.action });
    });

    return () => {
      unlistenHideRadialMenuEvent.then(f => f());
    }
  }, [items]);

  return (
    <>
      <div
        style={{
          position: 'absolute',
          width: `${radialMenuSize}px`,
          height: `${radialMenuSize}px`,
          background: `radial-gradient(circle, transparent ${threshold}px, rgba(0,0,0,0.5) ${threshold}px, rgba(0,0,0,0.5) 100%)`,
          borderRadius: '50%',
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
        }}
      >
        {items.map((e, i) => {
          const radius = radialMenuSize / 2;
          const innerRadius = threshold;
          const buttonRadius = (radius + innerRadius) / 2; // Place buttons in the middle of the band

          const angle = (2 * Math.PI * i) / items.length - Math.PI / 2; // Start from top
          const x = radius + buttonRadius * Math.cos(angle);
          const y = radius + buttonRadius * Math.sin(angle);

          return (
            <button
              key={i}
              onClick={() => {
                // Handle the action for the item
                console.log(`Action for ${e.label}:`, e.action);
              }}
              style={{
                position: 'absolute',
                left: `${x}px`,
                top: `${y}px`,
                transform: 'translate(-50%, -50%)',
                width: 'auto',
                height: 'auto',
                background: 'none',
                border: 'none',
                boxShadow: 'none',
                borderRadius: 0,
                color: 'white',
                fontSize: '16px',
                padding: 0,
                cursor: 'pointer',
                userSelect: 'none',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                pointerEvents: 'auto',
              }}
            >
              {e.label}
            </button>
          );
        })}
      </div>
    </>
  );
}
