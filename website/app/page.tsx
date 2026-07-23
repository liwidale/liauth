import { Faq } from "@/components/Faq";
import { FeatureGrid } from "@/components/FeatureGrid";
import { Hero } from "@/components/Hero";
import { OpenSource } from "@/components/OpenSource";
import { Philosophy } from "@/components/Philosophy";
import { SecurityArchitecture } from "@/components/SecurityArchitecture";
import { SiteFooter } from "@/components/SiteFooter";
import { SiteHeader } from "@/components/SiteHeader";
import { GITHUB_URL } from "@/lib/site";

const jsonLd = {
  "@context": "https://schema.org",
  "@type": "SoftwareApplication",
  name: "LiAuth",
  applicationCategory: "SecurityApplication",
  operatingSystem: "Android, iOS, macOS, Windows",
  offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
  license: "https://opensource.org/licenses/MIT",
  url: GITHUB_URL,
  description:
    "A modern, open-source authenticator for Android, iOS, macOS and Windows. Offline-first, end-to-end encrypted, built around a shared Rust core.",
};

export default function Home() {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
      />
      <SiteHeader />
      <div className="mx-auto max-w-[1120px] border-x border-line">
        <main>
          <Hero />
          <FeatureGrid />
          <SecurityArchitecture />
          <Philosophy />
          <OpenSource />
          <Faq />
        </main>
        <SiteFooter />
      </div>
    </>
  );
}
