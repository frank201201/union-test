<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"
import type { QueryBankBalanceError } from "$lib/services/cosmos/balances"
import type { FetchNativeBalanceError, ReadContractError } from "$lib/services/evm/balances"
import type { NoViemChainError } from "$lib/services/evm/clients"
import type { CreatePublicClientError } from "$lib/services/transfer/errors"
import type { Base64EncodeError } from "$lib/utils/base64"
import type { HttpClientError } from "@effect/platform/HttpClientError"
import type { NoRpcError } from "@unionlabs/sdk/schema"
import { extractErrorDetails } from "@unionlabs/sdk/utils"
import type { TimeoutException, UnknownException } from "effect/Cause"
import type { ParseError } from "effect/ParseResult"
import { slide } from "svelte/transition"
import BaselineCloseIcon from "../icons/BaselineCloseIcon.svelte"
import SharpDownloadIcon from "../icons/SharpDownloadIcon.svelte"
import SharpErrorOutlineIcon from "../icons/SharpErrorOutlineIcon.svelte"
import SharpOpenInBrowserIcon from "../icons/SharpOpenInBrowserIcon.svelte"
import Modal from "../ui/Modal.svelte"
import Tooltip from "../ui/Tooltip.svelte"

interface Props {
  error:
    | UnknownException
    | HttpClientError
    | ParseError
    | TimeoutException
    | NoViemChainError
    | ReadContractError
    | FetchNativeBalanceError
    | CreatePublicClientError
    | QueryBankBalanceError
    | Base64EncodeError
    | NoRpcError
  onClose?: (() => void) | undefined
}

let { error, onClose }: Props = $props()
let showDetails = $state(false)
let visible = $state(true)

// TODO: replace me with an exhaustive matcher :)
function getUserFriendlyMessage(error: Props["error"]): string {
  switch (error._tag) {
    case "RequestError":
      return "Unable to connect to the server. Please check your internet connection."
    case "ResponseError":
      return "The server encountered an error processing your request."
    case "ParseError":
      return "There was an error processing the data from the server."
    case "TimeoutException":
      return "The request timed out because it took too long. Please try again."
    case "UnknownException":
      return "An unexpected error occurred."
    case "NoViemChain":
      return "Chain configuration not found for the selected network."
    case "ReadContractError":
      return "Failed to read contract data from the network."
    case "FetchNativeBalanceError":
      return "Failed to fetch native token balance."
    case "CreatePublicClientError":
      return "Failed to create network connection."
    case "QueryBankBalanceError":
      return "Failed to query bank balance from the network."
    case "Base64EncodeError":
      return "Failed to encode query parameters."
    case "NoRpcError":
      return `No ${error.type} endpoint available for ${error.chain.display_name}.`
    default:
      return "Something went wrong. Please try again later."
  }
}

const writeToClipboard = () => {
  navigator.clipboard.writeText(JSON.stringify(extractErrorDetails(error), null, 2))
}

const exportData = () => {
  const datetime = new Date().toISOString().replace(/-|:|\.\d+/g, "")
  const data = JSON.stringify(extractErrorDetails(error), null, 2)
  const blob = new Blob([data], { type: "application/json" })
  const url = window.URL.createObjectURL(blob)
  const anchor = document.createElement("a")
  anchor.href = url
  anchor.download = `union-log-${datetime}.json`
  anchor.click()
  window.URL.revokeObjectURL(anchor.href)
}
</script>

{#if visible}
  <div class="p-4 rounded bg-zinc-925 border-2 border-red-500 overflow-hidden flex flex-col">
    {#if onClose}
      <div class="flex flex-row mb-2">
        <SharpErrorOutlineIcon class="text-red-500 size-4 min-w-4" />
        <div class="grow"></div>
        <Button
          class="self-end p-0 h-4"
          variant="outline"
          onclick={onClose}
        >
          <BaselineCloseIcon
            height="1rem"
            width="1rem"
          />
        </Button>
      </div>
    {/if}
    <div class="flex justify-between items-center gap-2">
      {#if !onClose}
        <SharpErrorOutlineIcon class="text-red-500 size-4 min-w-4" />
      {/if}
      <p>{getUserFriendlyMessage(error)}</p>
      <div class="grow"></div>
      <Tooltip delay={"quick"}>
        {#snippet trigger()}
          <Button
            variant="secondary"
            onclick={() => (showDetails = !showDetails)}
          >
            <SharpOpenInBrowserIcon class="size-4" />
          </Button>
        {/snippet}

        {#snippet content()}
          Open Details
        {/snippet}
      </Tooltip>
      <!-- <Tooltip delay={"quick"}>
      {#snippet trigger()}
        <Button variant="secondary" onclick={writeToClipboard}>
          <SharpContentCopyIcon class="size-4" />
        </Button>
      {/snippet}
      {#snippet content()}
        Copy to Clipboard
      {/snippet}
    </Tooltip> -->
      <Tooltip delay={"quick"}>
        {#snippet trigger()}
          <Button
            variant="primary"
            onclick={exportData}
          >
            <SharpDownloadIcon class="size-4" />
          </Button>
        {/snippet}
        {#snippet content()}
          Download Log
        {/snippet}
      </Tooltip>
    </div>

    <Modal
      isOpen={showDetails}
      onClose={() => (showDetails = !showDetails)}
      class="w-full max-w-4xl"
    >
      <div
        class="overflow-auto mt-6"
        in:slide
        out:slide|local={{ delay: 0 }}
      >
        <section class="mt-4">
          <h3 class="text-lg font-bold">Error Type</h3>
          <pre>{error._tag}</pre>
          <pre class="mt-2">{error.message}</pre>
        </section>

        {#if error.cause}
          <section class="mt-4">
            <h3 class="text-lg font-bold">Cause</h3>
            <pre>{error.cause}</pre>
          </section>
        {/if}

        {#if error.stack}
          <section class="mt-4">
            <h3 class="text-lg font-bold">Stack</h3>
            <pre class="text-sm">{error.stack}</pre>
          </section>
        {/if}

        <section class="mt-4">
          <h3 class="text-lg font-bold">Additional Details</h3>
          {#if error._tag === "RequestError"}
            <p>{error.description}</p>
            <p>Method and URL: {error.methodAndUrl}</p>
          {:else if error._tag === "ResponseError"}
            <p>{error.description}</p>
            <p>Method and URL: {error.methodAndUrl}</p>
          {:else if error._tag === "ParseError"}
            <p>Actual data that was parsed:</p>
            <pre
              class="text-sm"
            >
{JSON.stringify(
                error.issue.actual,
                null,
                2,
              )}</pre
            >
          {:else if error._tag === "UnknownException"}
            <p>This is an unknown exception. Full details here:</p>
            <pre class="text-sm">{JSON.stringify(error, null, 2)}</pre>
          {:else if error._tag === "NoViemChain"}
            <p>Chain ID: {error.chain.chain_id}</p>
            <p>Universal Chain ID: {error.chain.universal_chain_id}</p>
          {:else if error._tag === "ReadContractError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "FetchNativeBalanceError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "CreatePublicClientError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "QueryBankBalanceError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "Base64EncodeError"}
            <p>Error cause:</p>
            <pre class="text-sm">{JSON.stringify(error.cause, null, 2)}</pre>
          {:else if error._tag === "NoRpcError"}
            <p>Chain: {error.chain.display_name}</p>
            <p>RPC Type: {error.type}</p>
            <p>Available RPC types:</p>
            <pre
              class="text-sm"
            >
{JSON.stringify(
                error.chain.rpcs.map((r) => r.type),
                null,
                2,
              )}</pre
            >
          {/if}
        </section>
      </div>
    </Modal>
  </div>
{/if}
