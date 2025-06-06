import type { FetchDecodeGraphqlError } from "$lib/utils/queries"
import type { TransferDetails } from "@unionlabs/sdk/schema"
import { Option } from "effect"

class TransferDetailsStore {
  data = $state(Option.none<TransferDetails>())
  error = $state(Option.none<FetchDecodeGraphqlError | { _tag: "NotFound"; message: string }>())
}

export const transferDetails = new TransferDetailsStore()
