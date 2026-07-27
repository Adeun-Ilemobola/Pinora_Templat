import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import App from "./App";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "sonner";
createRoot(document.getElementById("root")!).render(
	<StrictMode>
		<TooltipProvider>
			<App />
			<Toaster richColors position="bottom-right" />
		</TooltipProvider>
		
	</StrictMode>,
);   
