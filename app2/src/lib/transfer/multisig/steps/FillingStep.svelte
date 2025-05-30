<script lang="ts">
import InsetError from "$lib/components/model/InsetError.svelte"
import Button from "$lib/components/ui/Button.svelte"
import Input from "$lib/components/ui/Input.svelte"
import Amount from "$lib/transfer/shared/components/Amount.svelte"
import ChainAsset from "$lib/transfer/shared/components/ChainAsset/index.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import type { ContextFlowError } from "$lib/transfer/shared/errors"
import { Match, Option } from "effect"
import ReceiverInput from "../components/ReceiverInput.svelte"
import SenderInput from "../components/SenderInput.svelte"

type Props = {
  onContinue: () => void
  loading: boolean
  onErrorClose?: () => void
  statusMessage?: string
  errors?: Option.Option<ContextFlowError>
}

const {
  onContinue,
  loading,
  statusMessage,
  errors = Option.none<ContextFlowError>(),
}: Props = $props()

let isModalOpen = $state(false)

const uiStatus = $derived.by(() => {
  return Option.match(errors, {
    onSome: error => {
      const match = Match.type<ContextFlowError>().pipe(
        Match.tag("BalanceLookupError", () => ({
          text: "Failed checking balance",
          error,
        })),
        Match.tag("AllowanceCheckError", () => ({
          text: "Failed checking allowance",
          error,
        })),
        Match.tag("OrderCreationError", () => ({
          text: "Could not create orders",
          error,
        })),
        Match.orElse(() => ({
          text: statusMessage ?? "Continue",
          error,
        })),
      )
      return match(error)
    },

    onNone: () => ({
      text: statusMessage ?? "Continue",
      error: null,
    }),
  })
})

const isButtonEnabled = $derived.by(() => !loading)
</script>

<div class="min-w-full flex flex-col grow">
  <div class="flex flex-col gap-4 p-4">
    <SenderInput />
    <ChainAsset type="source" />
    <ChainAsset type="destination" />
    <ReceiverInput />
    <Amount type="source" />
  </div>

  <div class="grow"></div>

  <div class="p-4 flex justify-between gap-2 border-t border-zinc-800 sticky bottom-0 bg-zinc-925">
    <div class="w-full items-end flex gap-2">
      {#if Option.isSome(errors)}
        <Button
          class="flex-1"
          variant="danger"
          onclick={() => (isModalOpen = true)}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {:else}
        <Button
          class="flex-1"
          variant="primary"
          onclick={onContinue}
          disabled={!isButtonEnabled}
        >
          {uiStatus.text}
        </Button>
      {/if}
    </div>
  </div>
</div>

<InsetError
  open={isModalOpen}
  error={uiStatus.error}
  onClose={() => (isModalOpen = false)}
/>
