<script lang="ts">
  import * as api from "../../api";
  import Pane from "../Pane.svelte";
  import GlassCard from "../../components/GlassCard.svelte";
  import SettingsRow from "../../components/SettingsRow.svelte";
  import Segmented from "../../components/Segmented.svelte";

  let { settings }: { settings: api.Settings } = $props();

  const blurb: Record<api.Method, string> = {
    telex: "Dấu bằng chữ cái — aa→â, ow→ơ, as→á, dd→đ",
    vni: "Dấu bằng chữ số — a6→â, o7→ơ, a1→á, d9→đ",
  };

  const toneBlurb: Record<api.ToneStyle, string> = {
    traditional: "Dấu kiểu cũ — hòa, khỏe, thúy",
    modern: "Dấu kiểu mới — hoà, khoẻ, thuý",
  };

  function pick(m: api.Method) {
    settings.method = m;
    api.setMethod(m);
  }

  function pickTone(t: api.ToneStyle) {
    settings.toneStyle = t;
    api.setToneStyle(t);
  }
</script>

<Pane title="Kiểu gõ">
  <GlassCard>
    <SettingsRow title="Phương thức" subtitle={blurb[settings.method]}>
      {#snippet control()}
        <Segmented
          options={[
            { id: "telex", label: "Telex" },
            { id: "vni", label: "VNI" },
          ]}
          value={settings.method}
          onchange={pick}
        />
      {/snippet}
    </SettingsRow>
  </GlassCard>

  <GlassCard>
    <SettingsRow title="Kiểu đặt dấu" subtitle={toneBlurb[settings.toneStyle]}>
      {#snippet control()}
        <Segmented
          options={[
            { id: "traditional", label: "Truyền thống" },
            { id: "modern", label: "Hiện đại" },
          ]}
          value={settings.toneStyle}
          onchange={pickTone}
        />
      {/snippet}
    </SettingsRow>
  </GlassCard>

  <GlassCard>
    <div class="try-label">Gõ thử</div>
    <input class="try" placeholder="Gõ ở đây…" />
    <p class="hint">
      Bản xem trước giao diện — gõ tiếng Việt thật ở mọi app sau khi Funput đang chạy.
    </p>
  </GlassCard>
</Pane>

<style>
  .try-label {
    font-size: 13px;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: var(--space-sm);
  }
  .try {
    width: 100%;
    padding: 10px 12px;
    border-radius: var(--radius-control);
    border: 1px solid var(--hairline);
    background: var(--glass-bg);
    color: var(--text);
    font-size: 14px;
    user-select: text;
  }
  .hint {
    font-size: 12px;
    color: var(--text-secondary);
    margin: var(--space-sm) 0 0;
  }
</style>
