import { createBrowserRouter } from "react-router-dom";
import { AppLayout } from "./components/AppLayout";
import { Home } from "./routes/Home";
import { BrowserConfig } from "./routes/BrowserConfig";
import { Automations } from "./routes/Automations";
import { Integrations } from "./routes/Integrations";
import { Tasks } from "./routes/Tasks";
import { LlmIntegrations } from "./routes/LlmIntegrations";
import { CaptchaResolvers } from "./routes/CaptchaResolvers";

export const router = createBrowserRouter([
  {
    path: "/",
    element: <AppLayout />,
    children: [
      { index: true, element: <Home /> },
      { path: "browser-config", element: <BrowserConfig /> },
      { path: "automations", element: <Automations /> },
      { path: "integrations", element: <Integrations /> },
      { path: "tasks", element: <Tasks /> },
      { path: "llm-integrations", element: <LlmIntegrations /> },
      { path: "captcha-resolvers", element: <CaptchaResolvers /> }
    ]
  }
]);
