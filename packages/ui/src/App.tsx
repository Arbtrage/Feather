import { Link, Route, Routes } from "react-router-dom";
import { JobDetailPage } from "./pages/JobDetailPage";
import { JobsPage } from "./pages/JobsPage";
import { OverviewPage } from "./pages/OverviewPage";

export default function App() {
  return (
    <>
      <header>
        <nav>
          <strong>Feather</strong>
          <Link to="/">Overview</Link>
          <Link to="/jobs">Jobs</Link>
        </nav>
      </header>
      <main>
        <Routes>
          <Route path="/" element={<OverviewPage />} />
          <Route path="/jobs" element={<JobsPage />} />
          <Route path="/jobs/:id" element={<JobDetailPage />} />
        </Routes>
      </main>
    </>
  );
}
