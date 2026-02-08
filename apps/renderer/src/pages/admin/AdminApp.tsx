import { Admin, Resource } from "react-admin"
import { incomeDataProvider } from "@/pages/admin/income-data-provider"
import { IncomeCreate, IncomeEdit, IncomeList } from "@/pages/admin/income-resource"

export function AdminApp() {
	return (
		<Admin dataProvider={incomeDataProvider} title="DCS会計 Admin">
			<Resource
				name="incomes"
				list={IncomeList}
				edit={IncomeEdit}
				create={IncomeCreate}
			/>
		</Admin>
	)
}
