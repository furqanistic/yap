import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/app/App";
import { ToastProvider } from "@/components/ui";
import { applyPlatformAttributes } from "@/lib/platform";
import "@/styles/global.css";

applyPlatformAttributes();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ToastProvider>
      <App />
    </ToastProvider>
  </React.StrictMode>,
);
