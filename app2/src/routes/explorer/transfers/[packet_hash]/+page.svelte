<script lang="ts">
import { page } from "$app/state"
import AddressComponent from "$lib/components/model/AddressComponent.svelte"
import ChainComponent from "$lib/components/model/ChainComponent.svelte"
import ErrorComponent from "$lib/components/model/ErrorComponent.svelte"
import PacketComponent from "$lib/components/model/PacketComponent.svelte"
import TokenComponent from "$lib/components/model/TokenComponent.svelte"
import Card from "$lib/components/ui/Card.svelte"
import DateTimeComponent from "$lib/components/ui/DateTimeComponent.svelte"
import Label from "$lib/components/ui/Label.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { packetDetailsQuery } from "$lib/queries/packet-details.svelte"
import { transferByPacketHashQuery } from "$lib/queries/transfer-by-hash.svelte"
import { chains } from "$lib/stores/chains.svelte"
import { packetDetails } from "$lib/stores/packets.svelte"
import { settingsStore } from "$lib/stores/settings.svelte"
import { cosmosStore } from "$lib/wallet/cosmos"
import { getChain, PacketHash, TokenRawDenom } from "@unionlabs/sdk/schema"
import { Effect, Fiber, Option, pipe, Schema, Struct } from "effect"
import { onMount } from "svelte"

// Store for the transfer details
import SharpCheckIcon from "$lib/components/icons/SharpCheckIcon.svelte"
import SharpWarningIcon from "$lib/components/icons/SharpWarningIcon.svelte"
import SpinnerIcon from "$lib/components/icons/SpinnerIcon.svelte"
import PacketTracesComponent from "$lib/components/model/PacketTracesComponent.svelte"
import A from "$lib/components/ui/A.svelte"
import Button from "$lib/components/ui/Button.svelte"
import { finalityDelays, settlementDelays } from "$lib/constants/settlement-times.ts"
import { transferDetails } from "$lib/stores/transfer-details.svelte"
import { fromHex } from "viem"
import type { PageData } from "./$types"

type Props = {
  data: PageData
}

const { data }: Props = $props()

// State for packet details visibility
let showPacketDetails = $state(false)

let fiber: Fiber.Fiber<any, any>

onMount(() => {
  fiber = Effect.runFork(transferByPacketHashQuery(data.packetHash))
  packetDetails.runEffect(packetDetailsQuery(data.packetHash))

  return async () => {
    await Effect.runPromise(Fiber.interrupt(fiber))
    transferDetails.data = Option.none()
    transferDetails.error = Option.none()

    // Clean up packet details
    packetDetails.interruptFiber()
  }
})

const inProgress = $derived(
  transferDetails.data.pipe(
    Option.map(Struct.get("traces")),
    Option.map(
      traces => !traces.some(t => t.type === "WRITE_ACK" && Option.isSome(t.transaction_hash)),
    ),
    Option.getOrElse(() => true),
  ),
)

const suggestTokenToWallet = async (chain_id: string, denom: TokenRawDenom) => {
  console.log("suggest token", chain_id, denom)
  const denomCosmwasm = fromHex(denom, "string")

  if (window.keplr) {
    console.log("adding to keplr")
    const x = await window.keplr.suggestToken(chain_id, denomCosmwasm)
    console.log(x)
  }

  if (window.leap) {
    console.log("adding to leap")
    const x = await window.leap.suggestCW20Token(chain_id, denomCosmwasm)
    console.log(x)
  }
}
</script>

<Sections>
  <Card
    class="overflow-auto"
    divided
  >
    <div class="p-4">Transfer Details</div>
    <div>
      {#if Option.isSome(transferDetails.error)}
        <ErrorComponent error={transferDetails.error.value} />
      {:else if Option.isSome(transferDetails.data) && Option.isSome(chains.data)}
        {@const transfer = transferDetails.data.value}
        {@const chainsList = chains.data.value}
        {@const sourceChain = getChain(
          chainsList,
          transfer.source_chain.universal_chain_id,
        )}
        {@const destChain = getChain(
          chainsList,
          transfer.destination_chain.universal_chain_id,
        )}

        <div class="space-y-8">
          <!-- Chain and Token Transfer Display -->
          <div class="flex flex-col gap-6">
            <div class="flex flex-1 items-center justify-between pt-6 px-4">
              <div class="text-2xl">
                {#if Option.isSome(destChain)}
                  <TokenComponent
                    chain={destChain.value}
                    denom={transfer.quote_token}
                    amount={transfer.quote_amount}
                  />
                {/if}
              </div>
              {#if !inProgress}
                <div class="flex items-center gap-2">
                  {#if Option.isSome(transfer.success)}
                    {#if transfer.success.value}
                      <SharpCheckIcon class="size-6 text-accent" />
                      <p class="text-babylon">Received</p>
                    {:else}
                      <SharpWarningIcon class="size-6 text-yellow-500 self-center" />
                      <p class="text-babylon">Failed transfer. Will be refunded.</p>
                    {/if}
                  {:else}
                    <SharpWarningIcon
                      class="text-yellow-500 self-center"
                      height="3rem"
                      width="3rem"
                    />
                    <p class="text-babylon">Received with unknown status</p>
                  {/if}
                </div>
              {:else}
                <div class="flex items-center gap-4">
                  <SpinnerIcon class="size-6" />
                  <p>In progress</p>
                </div>
              {/if}
            </div>
            <section class="flex flex-col px-4">
              <Label>From</Label>
              {#if Option.isSome(sourceChain)}
                {#if settingsStore.showQuoteTokens}
                  <TokenComponent
                    chain={sourceChain.value}
                    denom={transfer.base_token}
                    amount={transfer.base_amount}
                  />
                {/if}
                {#if Option.isSome(sourceChain)}
                  <ChainComponent chain={sourceChain.value} />
                  <AddressComponent
                    address={transfer.sender_canonical}
                    chain={sourceChain.value}
                    class="text-zinc-400"
                  />
                {:else}
                  <div>{transfer.source_chain.chain_id}</div>
                  <div class="font-mono text-sm text-zinc-400">
                    {transfer.sender_canonical}
                  </div>
                {/if}
              {/if}
              <DateTimeComponent
                class="text-sm text-zinc-400"
                value={transfer.transfer_send_timestamp}
                showSeconds={false}
              />
            </section>

            <section class="flex flex-col px-4">
              <Label>To</Label>
              {#if settingsStore.showQuoteTokens && Option.isSome(destChain)}
                <TokenComponent
                  chain={destChain.value}
                  denom={transfer.quote_token}
                  amount={transfer.quote_amount}
                />
              {/if}
              {#if Option.isSome(destChain)}
                <ChainComponent chain={destChain.value} />
                <AddressComponent
                  address={transfer.receiver_canonical}
                  chain={destChain.value}
                  class="text-zinc-400"
                />
              {:else}
                <div>{transfer.destination_chain.chain_id}</div>
                <div class="font-mono text-sm text-zinc-400">
                  {transfer.receiver_canonical}
                </div>
              {/if}
              {#if Option.isSome(transfer.transfer_recv_timestamp)}
                <DateTimeComponent
                  class="text-sm text-zinc-400"
                  value={transfer.transfer_recv_timestamp.value}
                  showSeconds={false}
                />
              {/if}
            </section>

            {#if Option.isSome(destChain) && destChain.value.rpc_type === "cosmos"}
              <section class="px-4">
                <Label>
                  Add to wallet
                </Label>
                <p class="text-sm">
                  First time transferring this asset? Make sure you have <b>Keplr</b> connected and
                  <Button
                    variant="inline"
                    onclick={() =>
                    suggestTokenToWallet(
                      destChain.value.chain_id,
                      transfer.quote_token,
                    )}
                  >
                    add it to your Keplr wallet.
                  </Button>
                </p>
                <p class="text-xs mt-4 text-zinc-300 italic">
                  If you do not see a popup, you have either not connected Keplr or already added it
                  to your wallet. In Keplr scroll down to "Manage Asset List" and re-enable the
                  token. Leap does not currently support this feature, but we are working with them
                  to add support soon.
                </p>
              </section>
            {/if}
            {#if Option.isSome(sourceChain)}
              {@const settlement = settlementDelays[sourceChain.value.universal_chain_id]}
              {#if settlement}
                <section class="flex flex-col px-4">
                  <Label>ETA</Label>
                  <p class="text-sm">
                    {sourceChain.value.display_name} is an L2. Outbound transfers are processed as
                    soon as
                    {sourceChain.value.display_name} settles (<A
                      class="underline"
                      href={settlement.url}
                    >happens every {settlement.interval}</A>).
                  </p>
                </section>
              {/if}
              {@const finality = finalityDelays[sourceChain.value.universal_chain_id]}
              {#if finality}
                <section class="flex flex-col px-4">
                  <Label>ETA</Label>
                  <p class="text-sm">
                    Transfers out of {sourceChain.value.display_name} are processed as soon as
                    {sourceChain.value.display_name} reaches finality. (<A
                      class="underline"
                      href={finality.url}
                    >approximately {finality.interval}</A>).
                  </p>
                </section>
              {/if}
            {/if}
          </div>

          <PacketTracesComponent
            packetTraces={transfer.traces}
            showAcks={false}
            mode="transfer"
          />
        </div>
      {/if}
    </div>
  </Card>

  <!-- Packet Details Card with toggle button -->
  <div>
    <button
      class="flex items-center justify-center w-full gap-2 py-2 px-4 text-left hover:text-zinc-300 text-zinc-400 cursor-pointer transition-colors"
      onclick={() => showPacketDetails = !showPacketDetails}
    >
      <span>Packet Details</span>
      <span
        class="transition-transform duration-300"
        style={showPacketDetails ? "transform: rotate(180deg)" : ""}
      >↓</span>
    </button>

    {#if showPacketDetails}
      <Card
        divided
        transition={false}
      >
        <PacketComponent />
      </Card>
    {/if}
  </div>
</Sections>
