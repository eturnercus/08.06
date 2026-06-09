import React from "react";
import ReactDOM from "react-dom/client";
import { migrateLegacyStorage } from "./utils/migrateLegacyStorage";
migrateLegacyStorage();
import "./i18n";
import "./styles/global.css";
import "./styles/material.css";
import "./styles/welcome.css";
import App from "./App";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
