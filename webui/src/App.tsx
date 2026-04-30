import { AppNavigation } from "@/components/app-navigation";
import { LoginPage } from "@/components/login-page";

export default function App() {
  return (
    <main className="min-h-screen bg-background text-foreground">
      <AppNavigation />
      <LoginPage />
    </main>
  );
}
