import json
import time
import logging
from selenium.webdriver.common.by import By
from selenium.webdriver.common.keys import Keys
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
import undetected_chromedriver as uc
from typing import Dict, Any, List, Optional

logging.basicConfig(level=logging.DEBUG, format='%(asctime)s %(levelname)s %(message)s')

class DebankScraper:
    def __init__(self, address: str):
        self.address = address
        options = uc.ChromeOptions()
        self.driver = uc.Chrome(options=options)
        self.driver.set_page_load_timeout(30)

    def close(self):
        self.driver.quit()

    def scrape(self) -> Dict[str, Any]:
        try:
            url = f"https://debank.com/profile/{self.address}"
            logging.info(f"Navigating to {url}")
            self.driver.get(url)
            self._wait_for_data_updated()
            self._expand_chains()
            self._log_chains_area_html()
            chain_rows = self.driver.find_elements(By.CSS_SELECTOR, 'div.AssetsOnChain_chainInfo__fKA2k')
            chains_data = {}
            for idx, row in enumerate(chain_rows):
                try:
                    chain_name = self._extract_chain_name(row)
                    logging.info(f"Clicando na chain: {chain_name}")
                    row.click()
                    time.sleep(1.2)
                    wallet_info = self._extract_wallet_info()
                    project_info = self._extract_project_info()
                    chains_data[chain_name] = {
                        'name': chain_name,
                        'wallet_info': wallet_info,
                        'project_info': project_info
                    }
                except Exception as e:
                    logging.warning(f"Erro ao processar chain {idx}: {e}")
            return chains_data
        finally:
            self.close()

    def _wait_for_data_updated(self):
        try:
            WebDriverWait(self.driver, 30).until(
                EC.visibility_of_element_located((By.CSS_SELECTOR, "div.HeaderInfo_totalAssetInner__HyrdC"))
            )
            WebDriverWait(self.driver, 30).until(
                EC.text_to_be_present_in_element((By.XPATH, "/html/body/div[1]/div[1]/div[1]/div/div/div/div[2]/div/div[2]/div[2]/span"), "Data updated")
            )
        except Exception as e:
            logging.warning(f"Timeout waiting for 'Data updated': {e}")

    def _expand_chains(self):
        try:
            logging.debug("Procurando botão de expandir cadeias...")
            expand_btn = self.driver.find_element(By.CSS_SELECTOR, '[class*="TotalAssetInfo_expandBtn__"]')
            expand_btn.click()
            logging.debug("Botão de expandir clicado, aguardando 2 segundos...")
            time.sleep(2)
        except Exception:
            logging.debug("Botão de expandir não encontrado ou já expandido.")
            pass

    def _log_chains_area_html(self):
        try:
            chains_area = self.driver.find_element(By.XPATH, '//div[contains(@class, "ChainList_chainList")]')
            logging.debug(f"HTML da área das chains:\n{chains_area.get_attribute('outerHTML')}")
        except Exception as e:
            logging.debug(f"Não foi possível extrair HTML da área das chains: {e}")

    def _extract_chain_name(self, row) -> str:
        try:
            chain_name_elem = row.find_element(By.XPATH, './div[1]')
        except Exception:
            try:
                chain_name_elem = row.find_element(By.CSS_SELECTOR, '[class*="ChainList_chainName__"], [class*="ChainList_chainName"]')
            except Exception:
                chain_name_elem = row.find_element(By.XPATH, './/div[contains(@class, "ChainList_chainName")]')
        return chain_name_elem.text.strip()

    def _extract_wallet_info(self) -> Dict[str, Any]:
        tokens = []
        try:
            wallet_container = self.driver.find_element(By.CSS_SELECTOR, 'div.TokenWallet_container__FUGTE')
            try:
                usd_value_elem = wallet_container.find_element(By.CSS_SELECTOR, '.projectTitle-number')
                usd_value = usd_value_elem.text.replace('$', '').replace(',', '').strip()
            except Exception:
                usd_value = None
            try:
                token_rows = wallet_container.find_elements(By.CSS_SELECTOR, '.db-table-wrappedRow')
                for roww in token_rows:
                    cols = roww.find_elements(By.CLASS_NAME, 'db-table-cell')
                    if len(cols) >= 4:
                        token_name = cols[0].text.strip()
                        price = cols[1].text.strip()
                        amount = cols[2].text.strip()
                        usd_value_token = cols[3].text.strip()
                        tokens.append({
                            'name': token_name,
                            'price': price,
                            'amount': amount,
                            'usd_value': usd_value_token
                        })
            except Exception:
                pass
            return {
                'usd_value': usd_value,
                'tokens': tokens
            }
        except Exception as e:
            logging.warning(f"Could not parse wallet info: {e}")
            return {}

    def _extract_project_info(self) -> List[Dict[str, Any]]:
        projects = self.driver.find_elements(By.CSS_SELECTOR, 'div.Project_project__GCrhx')
        project_info = []
        for pidx, project in enumerate(projects):
            try:
                name_elem = project.find_element(By.CSS_SELECTOR, '.ProjectTitle_protocolLink__4Yqn3')
                name = name_elem.text.strip()
                usd_value_elem = project.find_element(By.CSS_SELECTOR, '.projectTitle-number')
                usd_value = usd_value_elem.text.replace('$', '').replace(',', '').strip()
                logging.debug(f"[DEBUG] HTML do projeto {name} (pidx={pidx}):\n{project.get_attribute('outerHTML')}")
                panels = project.find_elements(By.CSS_SELECTOR, 'div.Panel_container__Vltd1')
                logging.debug(f"Projeto {name}: encontrados {len(panels)} painéis de tracking")
                if not panels:
                    logging.debug(f"[DEBUG] Nenhum painel encontrado para {name}. HTML completo:\n{project.get_attribute('outerHTML')}")
                trackings = []
                for panel in panels:
                    tracking_type = self._extract_tracking_type(panel)
                    tokens = self._extract_panel_tokens(panel)
                    # Agrupamento correto dos tokens em sections para Lending
                    if tracking_type == 'Lending':
                        sections = []
                        for section_title in ['Supplied', 'Borrowed', 'Rewards']:
                            section_tokens = [t for t in tokens if t.get('variant_header') == section_title]
                            sections.append({
                                'title': section_title,
                                'tokens': section_tokens
                            })
                        trackings.append({
                            'tracking_type': tracking_type,
                            'token_sections': sections
                        })
                    else:
                        trackings.append({
                            'tracking_type': tracking_type,
                            'token_sections': [
                                {
                                    'title': '<unused>',
                                    'tokens': tokens
                                }
                            ]
                        })
                project_info.append({
                    'name': name,
                    'trackings': trackings
                })
            except Exception as e:
                logging.debug(f"Could not parse project: {e}")
        return project_info

    def _extract_tracking_type(self, panel) -> Optional[str]:
        try:
            tracking_type_elem = panel.find_element(By.XPATH, './div[1]/div[1]/div[1]')
            return tracking_type_elem.text.strip()
        except Exception:
            return None

    def _extract_panel_tokens(self, panel) -> List[Dict[str, Any]]:
        import re
        tokens = []
        tables = panel.find_elements(By.XPATH, './div[2]/div')
        logging.debug(f"[DEBUG] Painel: encontrados {len(tables)} tabelas")
        for tidx, table in enumerate(tables):
            try:
                try:
                    header_row = table.find_element(By.XPATH, './div[1]')
                    headers = [h.text.strip() for h in header_row.find_elements(By.TAG_NAME, 'span')]
                except Exception:
                    headers = []
                logging.debug(f"[DEBUG] Tabela {tidx+1}: headers extraídos: {headers}")
                try:
                    body = table.find_element(By.XPATH, './div[2]')
                    token_rows = body.find_elements(By.CSS_SELECTOR, 'div.table_contentRow__Mi3k5.flex_flexRow__y0UR2')
                except Exception:
                    token_rows = []
                logging.debug(f"[DEBUG] Tabela {tidx+1}: {len(token_rows)} linhas de token encontradas")
                if not token_rows:
                    logging.warning(f"[DEBUG] Nenhuma linha de token encontrada na tabela {tidx+1}. HTML da tabela:\n{table.get_attribute('outerHTML')}")
                for row in token_rows:
                    cols = row.find_elements(By.TAG_NAME, 'div')
                    logging.debug(f"[DEBUG] Linha de token: {len(cols)} cols extraídos: {[c.text for c in cols]}")
                    if headers and len(cols) == len(headers):
                        token_data = {}
                        for i, header in enumerate(headers):
                            value = cols[i].text.strip() if i < len(cols) else ''
                            token_data[header] = value
                        # Pool: concatena todos os <a> da célula do pool
                        pool = None
                        if 'Pool' in headers:
                            pool_idx = headers.index('Pool')
                            pool_cell = cols[pool_idx]
                            pool_links = pool_cell.find_elements(By.TAG_NAME, 'a')
                            pool = '+'.join([a.text.strip() for a in pool_links if a.text.strip()])
                            if not pool:
                                pool = pool_cell.text.strip()
                        # Balance
                        balance = token_data.get('Balance')
                        # USD Value
                        usd_value = token_data.get('USD Value') or token_data.get('USD')
                        token_dict = {
                            'token_name': None,
                            'pool': pool,
                            'balance': balance,
                            'rewards': None,
                            'unlock_time': None,
                            'claimable_amount': None,
                            'end_time': None,
                            'usd_value': usd_value,
                            'variant_header': None
                        }
                        # Se for Lending, identificar variant_header
                        for header in ['Supplied', 'Borrowed', 'Rewards']:
                            if header in headers:
                                idx = headers.index(header)
                                value = cols[idx].text.strip()
                                token_dict['variant_header'] = header
                                if not token_dict['token_name']:
                                    token_dict['token_name'] = value
                                if header == 'Rewards':
                                    cleaned = re.sub(r'\(<?\$[0-9,.]+\)', '', value).strip()
                                    token_dict['rewards'] = cleaned
                                else:
                                    token_dict['rewards'] = None
                        if not token_dict['token_name']:
                            token_dict['token_name'] = pool
                        token_dict = {k: token_dict.get(k) for k in ['token_name','pool','balance','rewards','unlock_time','claimable_amount','end_time','usd_value','variant_header']}
                        logging.debug(f"[DEBUG] Token extraído (Rust-style): {token_dict}")
                        tokens.append(token_dict)
                    else:
                        # fallback antigo (pouco usado)
                        text_cols = [c.text.strip() for c in cols]
                        usd_value = next((t for t in reversed(text_cols) if t.startswith('$')), None)
                        pool = None
                        if len(cols) > 0:
                            pool_links = cols[0].find_elements(By.TAG_NAME, 'a')
                            pool = '+'.join([a.text.strip() for a in pool_links if a.text.strip()])
                            if not pool:
                                pool = cols[0].text.strip()
                        balance = next((t for t in text_cols if re.match(r'^[0-9,.]+$', t)), None)
                        token_dict = {
                            'token_name': pool,
                            'pool': pool,
                            'balance': balance,
                            'rewards': None,
                            'unlock_time': None,
                            'claimable_amount': None,
                            'end_time': None,
                            'usd_value': usd_value,
                            'variant_header': None
                        }
                        token_dict = {k: token_dict.get(k) for k in ['token_name','pool','balance','rewards','unlock_time','claimable_amount','end_time','usd_value','variant_header']}
                        logging.debug(f"[DEBUG] Token extraído (fallback): {token_dict}")
                        tokens.append(token_dict)
            except Exception as e:
                logging.debug(f'Erro ao processar tabela {tidx+1}: {e}')
        return tokens

def main():
    import sys
    address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    if len(sys.argv) == 2:
        address = sys.argv[1]
    try:
        scraper = DebankScraper(address)
        result = scraper.scrape()
        with open('debank_output.json', 'w', encoding='utf-8') as f:
            json.dump(result, f, indent=2, ensure_ascii=False)
        print(json.dumps(result, indent=2, ensure_ascii=False))
    except Exception as e:
        logging.error(f"Scraping failed: {e}")
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        exit(2)

if __name__ == "__main__":
    main()
