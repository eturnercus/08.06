import { useTranslation } from "react-i18next";
import {
  SectionTitle, SettingNumber, SettingSelect, SettingSlider,
  SettingText, SettingToggle,
} from "../ui/SettingField";

type Updater = (section: string, key: string, value: unknown) => void;
type Draft = Record<string, Record<string, unknown>>;

export function SystemPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const s = d.system;
  return (
    <>
      <SectionTitle>{t("settings.sections.hardware")}</SectionTitle>
      <SettingSlider title={t("settings.system.ramLimit")} desc={t("settings.system.ramLimitDesc")} value={s.ramLimitMb as number} onChange={(v) => u("system", "ramLimitMb", v)} min={512} max={131072} step={512} />
      <SettingSlider title={t("settings.system.ramSoft")} value={s.ramSoftLimitPercent as number} onChange={(v) => u("system", "ramSoftLimitPercent", v)} min={50} max={100} />
      <SettingNumber title={t("settings.system.vramReserve")} desc={t("settings.system.vramReserveDesc")} value={(s.vramReserveMb as number) ?? 512} onChange={(v) => u("system", "vramReserveMb", v)} />
      <SettingText title={t("settings.system.cpuCores")} desc={t("settings.system.cpuCoresDesc")} value={((s.cpuCores as number[] | undefined) ?? []).join(",")} onChange={(v) => u("system", "cpuCores", v.split(",").map(Number).filter((n) => !isNaN(n)))} />
      <SettingSelect title={t("settings.system.cpuAffinity")} value={s.cpuAffinityMode as string} options={["auto", "manual", "performance", "efficiency", "hybrid"]} onChange={(v) => u("system", "cpuAffinityMode", v)} />
      <SettingSelect
        title={t("settings.system.computeDevice")}
        desc={t("settings.system.computeDeviceDesc")}
        value={(s.computeDevice as string) || "auto"}
        options={[
          { v: "cpu", l: t("settings.system.computeCpu") },
          { v: "gpu", l: t("settings.system.computeGpu") },
          { v: "auto", l: t("settings.system.computeAuto") },
        ]}
        onChange={(v) => u("system", "computeDevice", v)}
      />
      <SettingNumber title={t("settings.system.gpuLayers")} desc={t("settings.system.gpuLayersDesc")} value={(s.gpuLayers as number) ?? 0} onChange={(v) => u("system", "gpuLayers", v)} />
      <SettingSlider title={t("settings.system.gpuMemory")} value={s.gpuMemoryMb as number} onChange={(v) => u("system", "gpuMemoryMb", v)} min={0} max={24576} step={256} />
      <SettingNumber title={t("settings.system.threads")} value={s.threadCount as number} onChange={(v) => u("system", "threadCount", v)} />
      <SettingNumber title={t("settings.system.batchSize")} value={s.batchSize as number} onChange={(v) => u("system", "batchSize", v)} />
      <SettingNumber title={t("settings.system.thermal")} value={(s.thermalThrottleC as number) ?? 85} onChange={(v) => u("system", "thermalThrottleC", v)} />
      <SectionTitle>{t("settings.sections.optimization")}</SectionTitle>
      <SettingToggle title={t("settings.system.mmap")} value={s.mmapEnabled as boolean} onChange={(v) => u("system", "mmapEnabled", v)} />
      <SettingToggle title={t("settings.system.mlock")} value={s.mlockEnabled as boolean} onChange={(v) => u("system", "mlockEnabled", v)} />
      <SettingToggle title={t("settings.system.hugePages")} value={(s.hugePages as boolean) ?? false} onChange={(v) => u("system", "hugePages", v)} />
      <SettingToggle title={t("settings.system.prefetchModels")} value={(s.prefetchModels as boolean) ?? true} onChange={(v) => u("system", "prefetchModels", v)} />
      <SettingSelect title={t("settings.system.priority")} value={s.processPriority as string} options={["idle", "low", "normal", "high", "realtime"]} onChange={(v) => u("system", "processPriority", v)} />
      <SettingSelect title={t("settings.system.ioPriority")} value={(s.ioPriority as string) ?? "normal"} options={["idle", "low", "normal", "high"]} onChange={(v) => u("system", "ioPriority", v)} />
      <SettingSelect title={t("settings.system.swap")} value={s.swapUsage as string} options={["none", "minimal", "aggressive"]} onChange={(v) => u("system", "swapUsage", v)} />
      <SettingSlider title={t("settings.system.diskCache")} value={s.diskCacheMb as number} onChange={(v) => u("system", "diskCacheMb", v)} min={0} max={16384} step={256} />
      <SettingSelect title={t("settings.system.oomPolicy")} value={s.oomPolicy as string} options={["kill", "graceful_degrade", "swap", "throttle"]} onChange={(v) => u("system", "oomPolicy", v)} />
    </>
  );
}

export function InnovationPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const inv = d.innovation ?? {};
  return (
    <>
      <div className="innovation-hero">
        <h3>🔮 {t("settings.innovation.title")}</h3>
        <p>{t("settings.innovation.desc")}</p>
      </div>
      <SectionTitle>{t("settings.innovation.cognitive")}</SectionTitle>
      <SettingToggle innovationHint title={t("settings.innovation.cognitiveLoadBalancer")} desc={t("settings.innovation.cognitiveLoadBalancerDesc")} value={(inv.cognitiveLoadBalancer as boolean) ?? true} onChange={(v) => u("innovation", "cognitiveLoadBalancer", v)} />
      <SettingSlider title={t("settings.innovation.cognitiveThreshold")} value={(inv.cognitiveLoadThreshold as number) ?? 0.75} onChange={(v) => u("innovation", "cognitiveLoadThreshold", v)} min={0} max={1} step={0.05} />
      <SettingToggle innovationHint title={t("settings.innovation.neuroplastic")} desc={t("settings.innovation.neuroplasticDesc")} value={(inv.neuroplasticMemory as boolean) ?? true} onChange={(v) => u("innovation", "neuroplasticMemory", v)} />
      <SettingSlider title={t("settings.innovation.adaptationRate")} value={(inv.neuroplasticAdaptationRate as number) ?? 0.05} onChange={(v) => u("innovation", "neuroplasticAdaptationRate", v)} min={0} max={1} step={0.01} />
      <SettingToggle innovationHint title={t("settings.innovation.synapticRouting")} desc={t("settings.innovation.synapticRoutingDesc")} value={(inv.synapticRouting as boolean) ?? true} onChange={(v) => u("innovation", "synapticRouting", v)} />
      <SettingSelect title={t("settings.innovation.synapticPriority")} value={(inv.synapticPathPriority as string) ?? "adaptive"} options={["shortest", "adaptive", "quality", "latency"]} onChange={(v) => u("innovation", "synapticPathPriority", v)} />
      <SectionTitle>{t("settings.innovation.context")}</SectionTitle>
      <SettingToggle innovationHint title={t("settings.innovation.contextDna")} desc={t("settings.innovation.contextDnaDesc")} value={(inv.contextDna as boolean) ?? true} onChange={(v) => u("innovation", "contextDna", v)} />
      <SettingSlider title={t("settings.innovation.dnaMutation")} value={(inv.contextDnaMutationRate as number) ?? 0.02} onChange={(v) => u("innovation", "contextDnaMutationRate", v)} min={0} max={0.5} step={0.01} />
      <SettingToggle innovationHint title={t("settings.innovation.holographic")} desc={t("settings.innovation.holographicDesc")} value={(inv.holographicContext as boolean) ?? true} onChange={(v) => u("innovation", "holographicContext", v)} />
      <SettingNumber title={t("settings.innovation.holographicDims")} value={(inv.holographicProjectionDims as number) ?? 512} onChange={(v) => u("innovation", "holographicProjectionDims", v)} />
      <SettingNumber title={t("settings.innovation.quantumLayers")} value={(inv.quantumContextLayers as number) ?? 4} onChange={(v) => u("innovation", "quantumContextLayers", v)} />
      <SettingSlider title={t("settings.innovation.quantumEntangle")} value={(inv.quantumEntanglementStrength as number) ?? 0.3} onChange={(v) => u("innovation", "quantumEntanglementStrength", v)} min={0} max={1} step={0.05} />
      <SectionTitle>{t("settings.innovation.streaming")}</SectionTitle>
      <SettingToggle innovationHint title={t("settings.innovation.thoughtStream")} desc={t("settings.innovation.thoughtStreamDesc")} value={(inv.thoughtStreaming as boolean) ?? true} onChange={(v) => u("innovation", "thoughtStreaming", v)} />
      <SettingNumber title={t("settings.innovation.streamBuffer")} value={(inv.thoughtStreamBufferMs as number) ?? 120} onChange={(v) => u("innovation", "thoughtStreamBufferMs", v)} />
      <SettingNumber title={t("settings.innovation.thoughtMaxTokens")} desc={t("settings.innovation.thoughtMaxTokensDesc")} value={(inv.thoughtMaxTokens as number) ?? 1024} onChange={(v) => u("innovation", "thoughtMaxTokens", v)} min={64} max={8192} />
      <SettingToggle innovationHint title={t("settings.innovation.attentionCascade")} value={(inv.attentionCascade as boolean) ?? true} onChange={(v) => u("innovation", "attentionCascade", v)} />
      <SettingNumber title={t("settings.innovation.cascadeDepth")} value={(inv.attentionCascadeDepth as number) ?? 6} onChange={(v) => u("innovation", "attentionCascadeDepth", v)} />
      <SectionTitle>{t("settings.innovation.advanced")}</SectionTitle>
      <SettingToggle innovationHint title={t("settings.innovation.emotionMirror")} value={(inv.emotionMirror as boolean) ?? false} onChange={(v) => u("innovation", "emotionMirror", v)} />
      <SettingSlider title={t("settings.innovation.emotionIntensity")} value={(inv.emotionMirrorIntensity as number) ?? 0.5} onChange={(v) => u("innovation", "emotionMirrorIntensity", v)} min={0} max={1} step={0.05} />
      <SettingToggle innovationHint title={t("settings.innovation.neuralMesh")} desc={t("settings.innovation.neuralMeshDesc")} value={(inv.neuralMeshSync as boolean) ?? false} onChange={(v) => u("innovation", "neuralMeshSync", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.dreamConsolidation")} desc={t("settings.innovation.dreamDesc")} value={(inv.dreamConsolidation as boolean) ?? true} onChange={(v) => u("innovation", "dreamConsolidation", v)} />
      <SettingSelect title={t("settings.innovation.dreamSchedule")} value={(inv.dreamConsolidationSchedule as string) ?? "idle"} options={["idle", "nightly", "manual"]} onChange={(v) => u("innovation", "dreamConsolidationSchedule", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.crossModal")} value={(inv.crossModalFusion as boolean) ?? true} onChange={(v) => u("innovation", "crossModalFusion", v)} />
      <SettingSlider title={t("settings.innovation.visionWeight")} value={(inv.crossModalWeightVision as number) ?? 0.4} onChange={(v) => u("innovation", "crossModalWeightVision", v)} min={0} max={1} step={0.05} />
      <SettingSlider title={t("settings.innovation.audioWeight")} value={(inv.crossModalWeightAudio as number) ?? 0.35} onChange={(v) => u("innovation", "crossModalWeightAudio", v)} min={0} max={1} step={0.05} />
      <SettingToggle innovationHint title={t("settings.innovation.predictivePrefetch")} value={(inv.predictivePrefetch as boolean) ?? true} onChange={(v) => u("innovation", "predictivePrefetch", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.neuralFirewall")} value={(inv.neuralFirewall as boolean) ?? true} onChange={(v) => u("innovation", "neuralFirewall", v)} />
      <SettingSlider title={t("settings.innovation.firewallSens")} value={(inv.firewallSensitivity as number) ?? 0.7} onChange={(v) => u("innovation", "firewallSensitivity", v)} min={0} max={1} step={0.05} />
      <SettingToggle innovationHint title={t("settings.innovation.metaCognition")} desc={t("settings.innovation.metaCognitionDesc")} value={(inv.metaCognitionLoop as boolean) ?? true} onChange={(v) => u("innovation", "metaCognitionLoop", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.echoBreaker")} desc={t("settings.innovation.echoBreakerDesc")} value={(inv.echoChamberBreaker as boolean) ?? true} onChange={(v) => u("innovation", "echoChamberBreaker", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.chronosync")} value={(inv.chronosyncMemory as boolean) ?? true} onChange={(v) => u("innovation", "chronosyncMemory", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.swarm")} value={(inv.swarmIntelligence as boolean) ?? false} onChange={(v) => u("innovation", "swarmIntelligence", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.latentNav")} value={(inv.latentSpaceNavigation as boolean) ?? true} onChange={(v) => u("innovation", "latentSpaceNavigation", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.whisperMode")} value={(inv.neuralWhisperMode as boolean) ?? false} onChange={(v) => u("innovation", "neuralWhisperMode", v)} />
      <SettingNumber title={t("settings.innovation.whisperBudget")} value={(inv.whisperTokenBudget as number) ?? 64} onChange={(v) => u("innovation", "whisperTokenBudget", v)} />
      <SettingNumber title={t("settings.innovation.prefetchHorizon")} value={(inv.prefetchHorizonTokens as number) ?? 256} onChange={(v) => u("innovation", "prefetchHorizonTokens", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.personaFluidity")} desc={t("settings.innovation.personaFluidityDesc")} value={(inv.personaFluidity as boolean) ?? false} onChange={(v) => u("innovation", "personaFluidity", v)} />
      <SettingSlider title={t("settings.innovation.personaBlend")} value={(inv.personaBlendRatio as number) ?? 0.3} onChange={(v) => u("innovation", "personaBlendRatio", v)} min={0} max={1} step={0.05} />
      <SettingToggle innovationHint title={t("settings.innovation.temporalAnchor")} desc={t("settings.innovation.temporalAnchorDesc")} value={(inv.temporalAnchoring as boolean) ?? true} onChange={(v) => u("innovation", "temporalAnchoring", v)} />
      <SettingNumber title={t("settings.innovation.temporalWindow")} value={(inv.temporalAnchorWindowMin as number) ?? 60} onChange={(v) => u("innovation", "temporalAnchorWindowMin", v)} />
      <SettingNumber title={t("settings.innovation.metaInterval")} value={(inv.metaCognitionInterval as number) ?? 10} onChange={(v) => u("innovation", "metaCognitionInterval", v)} />
      <SettingSlider title={t("settings.innovation.echoBoost")} value={(inv.echoDiversityBoost as number) ?? 0.25} onChange={(v) => u("innovation", "echoDiversityBoost", v)} min={0} max={1} step={0.05} />
      <SettingSelect title={t("settings.innovation.chronosyncGranularity")} value={(inv.chronosyncGranularity as string) ?? "message"} options={[
        { v: "message", l: t("settings.innovation.chronosyncMessage") },
        { v: "session", l: t("settings.innovation.chronosyncSession") },
        { v: "day", l: t("settings.innovation.chronosyncDay") },
      ]} onChange={(v) => u("innovation", "chronosyncGranularity", v)} />
      <SettingNumber title={t("settings.innovation.swarmParticles")} value={(inv.swarmParticleCount as number) ?? 8} onChange={(v) => u("innovation", "swarmParticleCount", v)} />
      <SettingNumber title={t("settings.innovation.latentSteps")} value={(inv.latentNavigationSteps as number) ?? 3} onChange={(v) => u("innovation", "latentNavigationSteps", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.ambientHarvest")} desc={t("settings.innovation.ambientHarvestDesc")} value={(inv.ambientContextHarvest as boolean) ?? false} onChange={(v) => u("innovation", "ambientContextHarvest", v)} />
      <SettingNumber title={t("settings.innovation.ambientInterval")} value={(inv.ambientHarvestIntervalSec as number) ?? 300} onChange={(v) => u("innovation", "ambientHarvestIntervalSec", v)} />
      <SettingToggle innovationHint title={t("settings.innovation.resonance")} desc={t("settings.innovation.resonanceDesc")} value={(inv.resonanceTuning as boolean) ?? false} onChange={(v) => u("innovation", "resonanceTuning", v)} />
      <SettingNumber title={t("settings.innovation.resonanceHz")} value={(inv.resonanceFrequencyHz as number) ?? 7.83} onChange={(v) => u("innovation", "resonanceFrequencyHz", v)} />
    </>
  );
}

export function PerformancePanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const p = d.performance ?? {};
  return (
    <>
      <SettingToggle title={t("settings.performance.turbo")} desc={t("settings.performance.turboDesc")} value={(p.turboMode as boolean) ?? false} onChange={(v) => u("performance", "turboMode", v)} />
      <SettingSlider title={t("settings.performance.turboBoost")} value={(p.turboRamBoostPercent as number) ?? 15} onChange={(v) => u("performance", "turboRamBoostPercent", v)} min={0} max={50} />
      <SettingToggle title={t("settings.performance.dynamicBatch")} value={(p.dynamicBatching as boolean) ?? true} onChange={(v) => u("performance", "dynamicBatching", v)} />
      <SettingToggle title={t("settings.performance.pipeline")} value={(p.pipelineParallelism as boolean) ?? true} onChange={(v) => u("performance", "pipelineParallelism", v)} />
      <SettingNumber title={t("settings.performance.pipelineStages")} value={(p.pipelineStages as number) ?? 4} onChange={(v) => u("performance", "pipelineStages", v)} />
      <SettingToggle title={t("settings.performance.continuousBatch")} value={(p.continuousBatching as boolean) ?? true} onChange={(v) => u("performance", "continuousBatching", v)} />
      <SettingToggle title={t("settings.performance.prefixCache")} value={(p.prefixCaching as boolean) ?? true} onChange={(v) => u("performance", "prefixCaching", v)} />
      <SettingToggle title={t("settings.performance.kvOffload")} value={(p.kvCacheOffload as boolean) ?? false} onChange={(v) => u("performance", "kvCacheOffload", v)} />
      <SettingSelect title={t("settings.performance.kvDevice")} value={(p.kvOffloadDevice as string) ?? "cpu"} options={["cpu", "gpu", "disk"]} onChange={(v) => u("performance", "kvOffloadDevice", v)} />
      <SettingSelect title={t("settings.performance.mixedPrecision")} value={(p.mixedPrecision as string) ?? "bf16"} options={["fp32", "fp16", "bf16", "int8", "int4"]} onChange={(v) => u("performance", "mixedPrecision", v)} />
      <SettingToggle title={t("settings.performance.compileGraph")} value={(p.compileGraph as boolean) ?? true} onChange={(v) => u("performance", "compileGraph", v)} />
      <SettingNumber title={t("settings.performance.warmup")} value={(p.warmupTokens as number) ?? 128} onChange={(v) => u("performance", "warmupTokens", v)} />
      <SettingToggle title={t("settings.performance.idlePower")} value={(p.idlePowerSave as boolean) ?? true} onChange={(v) => u("performance", "idlePowerSave", v)} />
      <SettingNumber title={t("settings.performance.latencyTarget")} value={(p.latencyTargetMs as number) ?? 200} onChange={(v) => u("performance", "latencyTargetMs", v)} />
      <SettingNumber title={t("settings.performance.queueDepth")} value={(p.maxQueueDepth as number) ?? 16} onChange={(v) => u("performance", "maxQueueDepth", v)} />
      <SettingNumber title={t("settings.performance.dynamicBatchMax")} value={(p.dynamicBatchMax as number) ?? 32} onChange={(v) => u("performance", "dynamicBatchMax", v)} />
      <SettingNumber title={t("settings.performance.prefixCacheTtl")} value={(p.prefixCacheTtlMin as number) ?? 60} onChange={(v) => u("performance", "prefixCacheTtlMin", v)} />
      <SettingNumber title={t("settings.performance.idleThreshold")} value={(p.idlePowerThresholdMin as number) ?? 5} onChange={(v) => u("performance", "idlePowerThresholdMin", v)} />
      <SettingToggle title={t("settings.performance.priorityQueue")} value={(p.priorityQueueInference as boolean) ?? true} onChange={(v) => u("performance", "priorityQueueInference", v)} />
      <SettingNumber title={t("settings.performance.tensorShards")} value={(p.tensorParallelShards as number) ?? 1} onChange={(v) => u("performance", "tensorParallelShards", v)} />
    </>
  );
}

export function SecurityPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const sec = d.security ?? {};
  return (
    <>
      <SettingToggle title={t("settings.security.encryptSettings")} value={(sec.encryptSettings as boolean) ?? false} onChange={(v) => u("security", "encryptSettings", v)} />
      <SettingToggle title={t("settings.security.encryptMemory")} value={(sec.encryptMemoryAtRest as boolean) ?? false} onChange={(v) => u("security", "encryptMemoryAtRest", v)} />
      <SettingToggle title={t("settings.security.confirmInternet")} value={(sec.requireConfirmationInternet as boolean) ?? true} onChange={(v) => u("security", "requireConfirmationInternet", v)} />
      <SettingToggle title={t("settings.security.confirmDevice")} value={(sec.requireConfirmationDevice as boolean) ?? true} onChange={(v) => u("security", "requireConfirmationDevice", v)} />
      <SettingToggle title={t("settings.security.auditLog")} value={(sec.auditLogEnabled as boolean) ?? true} onChange={(v) => u("security", "auditLogEnabled", v)} />
      <SettingToggle title={t("settings.security.sandbox")} value={(sec.sandboxProcessIsolation as boolean) ?? true} onChange={(v) => u("security", "sandboxProcessIsolation", v)} />
      <SettingToggle title={t("settings.security.promptShield")} value={(sec.promptInjectionShield as boolean) ?? true} onChange={(v) => u("security", "promptInjectionShield", v)} />
      <SettingSlider title={t("settings.security.shieldAggro")} value={(sec.shieldAggressiveness as number) ?? 0.6} onChange={(v) => u("security", "shieldAggressiveness", v)} min={0} max={1} step={0.05} />
      <SettingToggle title={t("settings.security.exfilGuard")} value={(sec.dataExfiltrationGuard as boolean) ?? true} onChange={(v) => u("security", "dataExfiltrationGuard", v)} />
      <SettingToggle title={t("settings.security.clipboard")} value={(sec.clipboardSanitization as boolean) ?? true} onChange={(v) => u("security", "clipboardSanitization", v)} />
      <SettingToggle title={t("settings.security.modelIntegrity")} value={(sec.modelIntegrityVerify as boolean) ?? true} onChange={(v) => u("security", "modelIntegrityVerify", v)} />
      <SettingNumber title={t("settings.security.autoLock")} value={(sec.autoLockMinutes as number) ?? 0} onChange={(v) => u("security", "autoLockMinutes", v)} />
      <SettingNumber title={t("settings.security.auditRetention")} value={(sec.auditLogRetentionDays as number) ?? 30} onChange={(v) => u("security", "auditLogRetentionDays", v)} />
      <SettingToggle title={t("settings.security.apiKeyVault")} value={(sec.apiKeyVault as boolean) ?? true} onChange={(v) => u("security", "apiKeyVault", v)} />
      <SettingToggle title={t("settings.security.networkFingerprint")} value={(sec.networkFingerprintCheck as boolean) ?? true} onChange={(v) => u("security", "networkFingerprintCheck", v)} />
    </>
  );
}

export function MemoryPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const m = d.memory;
  return (
    <>
      <SectionTitle>{t("settings.memory.stmSection")}</SectionTitle>
      <SettingToggle title={t("settings.memory.stmEnabled")} value={m.stmEnabled as boolean} onChange={(v) => u("memory", "stmEnabled", v)} />
      <SettingSlider title={t("settings.memory.stmMaxTokens")} value={m.stmMaxTokens as number} onChange={(v) => u("memory", "stmMaxTokens", v)} min={512} max={32768} step={512} />
      <SettingSlider title={t("settings.memory.stmMaxMessages")} desc={t("settings.memory.stmMaxMessagesDesc")} value={(m.stmMaxMessages as number) ?? 50} onChange={(v) => u("memory", "stmMaxMessages", v)} min={5} max={200} step={1} />
      <SettingNumber title={t("settings.memory.stmTtl")} value={m.stmTtlMinutes as number} onChange={(v) => u("memory", "stmTtlMinutes", v)} />
      <SettingNumber title={t("settings.memory.workingSlots")} value={(m.workingMemorySlots as number) ?? 7} onChange={(v) => u("memory", "workingMemorySlots", v)} />
      <SectionTitle>{t("settings.memory.ltmSection")}</SectionTitle>
      <SettingToggle title={t("settings.memory.ltmEnabled")} value={m.ltmEnabled as boolean} onChange={(v) => u("memory", "ltmEnabled", v)} />
      <SettingNumber title={t("settings.memory.ltmMaxEntries")} value={m.ltmMaxEntries as number} onChange={(v) => u("memory", "ltmMaxEntries", v)} />
      <SettingToggle title={t("settings.memory.episodic")} value={(m.episodicMemory as boolean) ?? true} onChange={(v) => u("memory", "episodicMemory", v)} />
      <SettingToggle title={t("settings.memory.procedural")} value={(m.proceduralMemory as boolean) ?? true} onChange={(v) => u("memory", "proceduralMemory", v)} />
      <SettingToggle title={t("settings.memory.graph")} value={(m.memoryGraphEnabled as boolean) ?? true} onChange={(v) => u("memory", "memoryGraphEnabled", v)} />
      <SettingSelect title={t("settings.memory.forgettingCurve")} value={(m.forgettingCurve as string) ?? "ebbinghaus"} options={["ebbinghaus", "linear", "exponential", "none"]} onChange={(v) => u("memory", "forgettingCurve", v)} />
      <SectionTitle>{t("settings.memory.transferSection")}</SectionTitle>
      <SettingSelect title={t("settings.memory.transferPolicy")} value={m.transferPolicy as string} options={["explicit_approval", "auto", "disabled"]} onChange={(v) => u("memory", "transferPolicy", v)} />
      <SettingToggle title={t("settings.memory.crossChat")} value={m.crossChatTransfer as boolean} onChange={(v) => u("memory", "crossChatTransfer", v)} />
      <SettingToggle title={t("settings.memory.crossModel")} value={m.crossModelTransfer as boolean} onChange={(v) => u("memory", "crossModelTransfer", v)} />
      <SettingToggle title={t("settings.memory.autoConsolidate")} value={m.autoConsolidate as boolean} onChange={(v) => u("memory", "autoConsolidate", v)} />
      <SettingToggle title={t("settings.memory.encryption")} value={m.memoryEncryption as boolean} onChange={(v) => u("memory", "memoryEncryption", v)} />
      <SettingToggle title={t("settings.memory.semanticSearch")} value={m.semanticSearch as boolean} onChange={(v) => u("memory", "semanticSearch", v)} />
      <SettingNumber title={t("settings.memory.recallTopK")} value={m.recallTopK as number} onChange={(v) => u("memory", "recallTopK", v)} />
    </>
  );
}

export function NetworkPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const n = d.network;
  return (
    <>
      <SettingSelect title={t("settings.network.isolation")} value={n.isolationMode as string} options={[
        { v: "full", l: t("settings.network.isolationFull") },
        { v: "api_only", l: t("settings.network.isolationApi") },
        { v: "none", l: t("settings.network.isolationNone") },
      ]} onChange={(v) => u("network", "isolationMode", v)} />
      <SettingToggle title={t("settings.network.allowInternet")} value={n.allowInternet as boolean} onChange={(v) => u("network", "allowInternet", v)} />
      <SettingToggle title={t("settings.network.offlineFallback")} value={(n.offlineFallback as boolean) ?? true} onChange={(v) => u("network", "offlineFallback", v)} />
      <SettingText title={t("settings.network.apiEndpoints")} value={((n.apiOnlyEndpoints as string[] | undefined) ?? []).join("\n")} onChange={(v) => u("network", "apiOnlyEndpoints", v.split("\n").filter(Boolean))} multiline />
      <SettingText title={t("settings.network.proxy")} value={n.proxyUrl as string} onChange={(v) => u("network", "proxyUrl", v)} />
      <SettingToggle title={t("settings.network.tor")} value={(n.torEnabled as boolean) ?? false} onChange={(v) => u("network", "torEnabled", v)} />
      <SettingSelect title={t("settings.network.egressFilter")} value={(n.egressFilterMode as string) ?? "strict"} options={["permissive", "standard", "strict", "paranoid"]} onChange={(v) => u("network", "egressFilterMode", v)} />
      <SettingNumber title={t("settings.network.rateLimit")} value={(n.rateLimitRpm as number) ?? 60} onChange={(v) => u("network", "rateLimitRpm", v)} />
      <SettingToggle title={t("settings.network.logRequests")} value={n.logAllRequests as boolean} onChange={(v) => u("network", "logAllRequests", v)} />
      <SettingToggle title={t("settings.network.blockPrivate")} value={n.blockPrivateIps as boolean} onChange={(v) => u("network", "blockPrivateIps", v)} />
      <SettingToggle title={t("settings.network.tls")} value={n.tlsVerify as boolean} onChange={(v) => u("network", "tlsVerify", v)} />
      <SettingToggle title={t("settings.network.doh")} value={n.dnsOverHttps as boolean} onChange={(v) => u("network", "doh", v)} />
    </>
  );
}

export function InjectionPanel({ d, u }: { d: Draft; u: Updater }) {
  const { t } = useTranslation();
  const inj = d.globalMessageInjection;
  return (
    <>
      <div className="innovation-hero">
        <h3>✨ {t("settings.injection.title")}</h3>
        <p>{t("settings.injection.desc")}</p>
      </div>
      <SettingToggle title={t("settings.injection.enabled")} value={inj.enabled as boolean} onChange={(v) => u("globalMessageInjection", "enabled", v)} />
      <SettingText title={t("settings.injection.systemPrefix")} desc={t("settings.injection.systemPrefixDesc")} value={inj.systemPrefix as string} onChange={(v) => u("globalMessageInjection", "systemPrefix", v)} multiline />
      <SettingText title={t("settings.injection.userSuffix")} value={inj.userSuffix as string} onChange={(v) => u("globalMessageInjection", "userSuffix", v)} multiline />
      <SettingText title={t("settings.injection.hiddenContext")} desc={t("settings.injection.hiddenContextDesc")} value={inj.hiddenContext as string} onChange={(v) => u("globalMessageInjection", "hiddenContext", v)} multiline />
      <SettingToggle title={t("settings.injection.injectMemory")} value={inj.injectMemorySummary as boolean} onChange={(v) => u("globalMessageInjection", "injectMemorySummary", v)} />
      <SettingToggle title={t("settings.injection.injectDevice")} value={inj.injectDeviceState as boolean} onChange={(v) => u("globalMessageInjection", "injectDeviceState", v)} />
      <SettingToggle title={t("settings.injection.injectTime")} value={inj.injectTimestamp as boolean} onChange={(v) => u("globalMessageInjection", "injectTimestamp", v)} />
      <SettingToggle title={t("settings.injection.injectLocale")} value={inj.injectLocale as boolean} onChange={(v) => u("globalMessageInjection", "injectLocale", v)} />
    </>
  );
}
