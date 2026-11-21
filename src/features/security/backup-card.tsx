import { CircleCheck, FolderOpen, Loader2, RotateCcw, Save } from "lucide-react"
import { useId, useState } from "react"
import { Button } from "@/components/ui/button"
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card"
import { Field } from "@/components/ui/field"
import { Checkbox, Input } from "@/components/ui/input"
import { useTranslations } from "@/i18n/use-translations"
import { useErrorMessage } from "@/lib/use-error-message"
import {
	type BackupFile,
	useBackupNow,
	useBackupSettings,
	useChooseBackupDir,
	useInspectBackup,
	useRestoreBackup,
	useSelectBackup,
} from "./security-api"

/** バイト数を MB で（小数 1 桁）。 */
export function formatSize(bytes: number) {
	return `${(bytes / 1024 / 1024).toFixed(1)} MB`
}

/**
 * データ: バックアップと復元。バックアップは帳簿と同じパスワードで暗号化した 1 つのファイルで、
 * 終了するときに自動で作る（新しい 7 回分を残す）。帳簿のデータは PC の外に送らない。
 */
export function BackupCard() {
	const t = useTranslations("backup")
	const errorMessage = useErrorMessage()
	const settings = useBackupSettings()
	const backupNow = useBackupNow()
	const chooseDir = useChooseBackupDir()
	const select = useSelectBackup()
	const restore = useRestoreBackup()
	const [chosen, setChosen] = useState<{
		name: string
		revision: number
	} | null>(null)
	const choosingDisabled = select.isPending || restore.isPending

	const pick = (name: string | null) => {
		select.mutate(name, {
			onSuccess: (picked) => {
				restore.reset()
				setChosen((previous) =>
					picked
						? { name: picked, revision: (previous?.revision ?? 0) + 1 }
						: null
				)
			},
		})
	}

	const data = settings.data
	return (
		<Card id="book-backup" tabIndex={-1}>
			<CardHeader>
				<CardTitle help={t("description")}>{t("title")}</CardTitle>
				<CardDescription>{t("restorePasswordNote")}</CardDescription>
			</CardHeader>
			<CardContent className="flex flex-col gap-5">
				{settings.error && (
					<p role="alert" className="text-destructive text-sm">
						{errorMessage(settings.error)}
					</p>
				)}
				{data && (
					<dl className="grid gap-x-6 gap-y-1.5 text-sm sm:grid-cols-[auto_1fr]">
						<dt className="text-muted-foreground">{t("folder")}</dt>
						<dd className="break-all">
							{data.dir}
							{data.isDefault && (
								<span className="ml-2 text-muted-foreground">
									{t("defaultFolder")}
								</span>
							)}
						</dd>
						<dt className="text-muted-foreground">{t("bookSize")}</dt>
						<dd className="num">{formatSize(data.bookSize)}</dd>
					</dl>
				)}
				<p className="text-muted-foreground text-xs">{t("sizeNote")}</p>
				<div className="flex flex-wrap gap-2">
					<Button
						onClick={() => backupNow.mutate()}
						disabled={backupNow.isPending}
					>
						{backupNow.isPending ? (
							<Loader2 aria-hidden="true" className="animate-spin" />
						) : (
							<Save aria-hidden="true" />
						)}
						{t("backupNow")}
					</Button>
					<Button
						variant="outline"
						onClick={() => chooseDir.mutate(false)}
						disabled={chooseDir.isPending}
					>
						<FolderOpen aria-hidden="true" />
						{t("changeFolder")}
					</Button>
					{data && !data.isDefault && (
						<Button variant="ghost" onClick={() => chooseDir.mutate(true)}>
							{t("resetFolder")}
						</Button>
					)}
				</div>
				{backupNow.isSuccess && (
					<p role="status" className="flex items-center gap-2 text-sm">
						<CircleCheck aria-hidden="true" className="size-4 text-success" />
						{t("saved", { name: backupNow.data })}
					</p>
				)}
				{(backupNow.error || chooseDir.error || select.error) && (
					<p role="alert" className="text-destructive text-sm">
						{errorMessage(backupNow.error ?? chooseDir.error ?? select.error)}
					</p>
				)}

				<div className="flex flex-col gap-2">
					<p className="font-bold text-sm">{t("listTitle")}</p>
					{data && data.files.length === 0 ? (
						<p className="text-muted-foreground text-sm">{t("empty")}</p>
					) : (
						<ul className="max-h-72 overflow-y-auto rounded-lg border text-sm">
							{data?.files.map((file) => (
								<BackupRow
									key={file.name}
									file={file}
									onRestore={() => pick(file.name)}
									selected={chosen?.name === file.name}
									disabled={choosingDisabled}
								/>
							))}
						</ul>
					)}
					<Button
						variant="outline"
						size="sm"
						className="w-fit"
						onClick={() => pick(null)}
						disabled={choosingDisabled}
					>
						<RotateCcw aria-hidden="true" />
						{t("restoreOther")}
					</Button>
				</div>

				{chosen && (
					<RestorePanel
						key={chosen.revision}
						name={chosen.name}
						choosing={select.isPending}
						restore={restore}
						onDone={() => setChosen(null)}
					/>
				)}
			</CardContent>
		</Card>
	)
}

function BackupRow({
	file,
	onRestore,
	selected,
	disabled,
}: {
	file: BackupFile
	onRestore: () => void
	selected: boolean
	disabled: boolean
}) {
	const t = useTranslations("backup")
	return (
		<li
			className={`flex flex-wrap items-center justify-between gap-3 border-b px-3 py-2 last:border-b-0 ${selected ? "bg-accent" : ""}`}
		>
			<span className="flex flex-col">
				<span className="num">{file.createdAt}</span>
				<span className="text-muted-foreground text-xs">
					{t(`kind_${file.kind}`)}・
					<span className="num">{formatSize(file.size)}</span>
				</span>
			</span>
			<Button
				variant="ghost"
				size="sm"
				disabled={disabled}
				onClick={onRestore}
				aria-label={t("restoreFrom", { date: file.createdAt })}
			>
				{t("restore")}
			</Button>
		</li>
	)
}

/** 選んだバックアップを確かめてから復元する。 */
function RestorePanel({
	name,
	choosing,
	restore,
	onDone,
}: {
	name: string
	choosing: boolean
	restore: ReturnType<typeof useRestoreBackup>
	onDone: () => void
}) {
	const t = useTranslations("backup")
	const listSeparator = useTranslations("app")("listSeparator")
	const errorMessage = useErrorMessage()
	const inspect = useInspectBackup()
	const sameId = useId()
	const [samePassword, setSamePassword] = useState(true)
	const [password, setPassword] = useState("")
	const key = samePassword ? null : password
	const busy = choosing || inspect.isPending || restore.isPending

	return (
		<section
			aria-label={t("restoreTitle")}
			className="flex flex-col gap-4 rounded-lg border p-4"
		>
			<p className="font-bold text-sm">
				{t("restoreTitle")}: <span className="font-normal">{name}</span>
			</p>
			<div className="flex items-center gap-2 text-sm">
				<Checkbox
					id={sameId}
					checked={samePassword}
					disabled={busy}
					onChange={(e) => {
						inspect.reset()
						restore.reset()
						setSamePassword(e.target.checked)
					}}
				/>
				<label htmlFor={sameId}>{t("samePassword")}</label>
			</div>
			{!samePassword && (
				<Field
					label={t("backupPassword")}
					hint={t("backupPasswordHint")}
					className="max-w-sm"
				>
					<Input
						type="password"
						autoComplete="off"
						value={password}
						disabled={busy}
						onChange={(e) => {
							inspect.reset()
							restore.reset()
							setPassword(e.target.value)
						}}
					/>
				</Field>
			)}
			<Button
				variant="outline"
				className="w-fit"
				onClick={() => inspect.mutate(key)}
				disabled={busy}
			>
				{t("check")}
			</Button>
			{inspect.error && (
				<p role="alert" className="text-destructive text-sm">
					{errorMessage(inspect.error)}
				</p>
			)}
			{inspect.data && !restore.isSuccess && (
				<div className="flex flex-col gap-3">
					<dl className="grid gap-x-6 gap-y-1 text-sm sm:grid-cols-[auto_1fr]">
						<dt className="text-muted-foreground">{t("summaryName")}</dt>
						<dd>{inspect.data.tradeName || "-"}</dd>
						<dt className="text-muted-foreground">{t("summaryYears")}</dt>
						<dd className="num">
							{inspect.data.fiscalYears.join(listSeparator) || "-"}
						</dd>
						<dt className="text-muted-foreground">
							{t("summaryTransactions")}
						</dt>
						<dd className="num">{inspect.data.transactions}</dd>
					</dl>
					<p className="rounded-lg border border-warning/40 bg-warning-background px-3 py-2 text-sm text-warning">
						{t("restoreWarning")}
					</p>
					{restore.error && (
						<p role="alert" className="text-destructive text-sm">
							{errorMessage(restore.error)}
						</p>
					)}
					<div className="flex gap-2">
						<Button
							variant="destructive"
							onClick={() => restore.mutate(key)}
							disabled={busy}
						>
							{restore.isPending && (
								<Loader2 aria-hidden="true" className="animate-spin" />
							)}
							{t("restoreButton")}
						</Button>
						<Button variant="ghost" onClick={onDone} disabled={busy}>
							{t("cancel")}
						</Button>
					</div>
				</div>
			)}
			{restore.isSuccess && (
				<div className="flex flex-col gap-2">
					<p role="status" className="flex items-center gap-2 text-sm">
						<CircleCheck aria-hidden="true" className="size-4 text-success" />
						{t("restored")}
					</p>
					<Button
						variant="outline"
						size="sm"
						className="w-fit"
						onClick={onDone}
					>
						{t("close")}
					</Button>
				</div>
			)}
		</section>
	)
}
